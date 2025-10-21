use std::mem::MaybeUninit;

/// A unique token representing the global uninitialized state.
///
/// This token is a capability that allows initialization of [`Place`] instances.
/// There can be at most one `InitPhase` instance in the entire process.
#[derive(Debug)]
pub struct InitPhase {
    _private: (),
}

/// A token representing that all [`Place`] instances are properly initialized.
///
/// This is a zero-sized type that serves as proof that initialization has completed.
/// All borrows from [`Place`] instances are tied to the lifetime of this token,
/// ensuring they cannot outlive the initialization phase.
///
/// There must be at most one `ActivePhase` instance in the entire process.
#[derive(Debug)]
pub struct ActivePhase {
    _private: (),
}

impl InitPhase {
    /// Attempts to create the unique program-wide initialization token.
    ///
    /// This function will return `Some(InitPhase)` exactly once per process.
    /// All subsequent calls return `None`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let init_token = InitPhase::take().expect("should only be called once");
    /// ```
    pub fn take() -> crate::Result<Self> {
        use std::sync::Once;
        static INIT: Once = Once::new();

        let mut result = None;

        INIT.call_once(|| {
            result = Some(InitPhase { _private: () });
        });

        match result {
            Some(token) => Ok(token),
            None => Err(crate::error::Error::DuplicatedInit("InitPhase")),
        }
    }

    /// Transitions from initialization phase to active phase.
    ///
    /// Consumes the `InitPhase` token and produces an `ActivePhase` token,
    /// signaling that all [`Place`] instances have been properly initialized.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that every [`Place`] that will be accessed
    /// during the active phase has been properly initialized by calling
    /// [`Place::initialize`] with a mutable reference to this `InitPhase`.
    ///
    /// Failure to initialize a [`Place`] before calling this method may result
    /// in undefined behavior when that place is accessed later.
    pub unsafe fn complete(self) -> ActivePhase {
        ActivePhase { _private: () }
    }
}

/// A storage location with managed initialization lifecycle.
///
/// This trait represents a place that can be:
/// 1. Initialized during the [`InitPhase`]
/// 2. Accessed during the [`ActivePhase`]
/// 3. Deinitialized during the [`DropPhase`]
///
/// # Safety
///
/// Implementors must ensure that:
/// - [`initialize`] properly initializes the underlying storage
/// - [`get`] and [`get_mut`] only return references to initialized data
/// - [`deinitialize`] safely drops the contained value
///
/// The trait itself is `unsafe` because implementors must uphold these
/// invariants, but the access methods are safe to call once the proper
/// phase tokens are provided.
///
/// [`initialize`]: Place::initialize
/// [`get`]: Place::get
/// [`get_mut`]: Place::get_mut
/// [`deinitialize`]: Place::deinitialize
pub trait Place<T> {
    /// Initializes this place with the given value.
    ///
    /// # Arguments
    ///
    /// * `init_token` - A mutable reference to the global [`InitPhase`] token
    ///
    /// # Returns
    ///  `MaybeUninit<T>` to be initialized.
    ///
    /// A mutable reference to the underlying storage, tied to the lifetime
    /// of the `init_token`.
    fn initialize<'a>(&'static self, _init_token: &'a mut InitPhase) -> &'a mut MaybeUninit<T>;

    /// Returns a shared reference to the contained value.
    ///
    /// The returned reference is valid for the lifetime of the [`ActivePhase`] token.
    fn get<'a>(&'static self, active_token: &'a ActivePhase) -> &'a T;

    /// Returns a mutable reference to the contained value.
    ///
    /// The returned reference is valid for the lifetime of the [`ActivePhase`] token.
    /// Callers must ensure they have exclusive access to this place.
    fn get_mut<'a>(&'static self, active_token: &'a mut ActivePhase) -> &'a mut T;

    /// Deinitializes (drops) the contained value.
    ///
    /// # Safety
    ///
    /// This method is `unsafe` because:
    /// - It must be called at most once per place
    /// - No references to this place may exist after calling
    /// - The place must not be used after deinitialization
    unsafe fn deinitialize(&'static self, drop_token: &mut ActivePhase);
}

// Implementation for StaticPlace remains largely the same but uses new names
impl<T> Place<T> for crate::place::StaticPlace<T> {
    fn initialize<'a>(&'static self, _init_token: &'a mut InitPhase) -> &'a mut MaybeUninit<T> {
        unsafe { &mut *self.as_mut_ptr() }
    }

    fn get<'a>(&'static self, _active_token: &'a ActivePhase) -> &'a T {
        unsafe { (*self.as_mut_ptr()).assume_init_ref() }
    }

    fn get_mut<'a>(&'static self, _active_token: &'a mut ActivePhase) -> &'a mut T {
        unsafe { (*self.as_mut_ptr()).assume_init_mut() }
    }

    unsafe fn deinitialize(&'static self, _drop_token: &mut ActivePhase) {
        unsafe { (*self.as_mut_ptr()).assume_init_drop() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::place::StaticPlace;

    struct TestStruct(u32, String, &'static str);

    impl Drop for TestStruct {
        fn drop(&mut self) {
            println!("Dropping TestStruct({}, {}, {})", self.0, self.1, self.2);
        }
    }

    #[test]
    fn lifecycle_management() {
        static STORAGE: StaticPlace<TestStruct> = StaticPlace::uninit();

        let mut init_phase = InitPhase::take().expect("should get init token once");

        // Initialize during init phase
        let value_ref =
            STORAGE
                .initialize(&mut init_phase)
                .write(TestStruct(42, "Hello".to_string(), "world"));
        assert_eq!(value_ref.0, 42);

        // Transition to active phase
        let mut active_phase = unsafe { init_phase.complete() };

        // Access during active phase
        let shared_ref: &TestStruct = STORAGE.get(&active_phase);
        assert_eq!(shared_ref.1, "Hello");

        let exclusive_ref: &mut TestStruct = STORAGE.get_mut(&mut active_phase);
        exclusive_ref.0 += 1;
        exclusive_ref.1.push_str(", universe!");

        // Transition to drop phase and deinitialize
        unsafe {
            STORAGE.deinitialize(&mut active_phase);
        }

        assert!(InitPhase::take().is_err());
    }
}
