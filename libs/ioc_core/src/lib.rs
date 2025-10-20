pub mod life;
pub mod place;

use std::ops::{Deref, DerefMut};

pub trait Alias<Name> {
    type Key;
}

pub trait Context: Deref<Target = Ctx> {
    #[inline(always)]
    fn get_by_alias<Name>(&self) -> &<Ctx as Registered<<Self as Alias<Name>>::Key>>::Bean
    where
        Self: Alias<Name>,
        Ctx: Registered<<Self as Alias<Name>>::Key>,
    {
        <Ctx as Registered<<Self as Alias<Name>>::Key>>::get(self)
    }

    #[inline(always)]
    fn get_by_key<K>(&self) -> &<Ctx as Registered<K>>::Bean
    where
        Ctx: Registered<K>,
    {
        <Ctx as Registered<K>>::get(self)
    }
}

impl<C> Context for C where C: Deref<Target = Ctx> {}

pub unsafe trait Registered<K: ?Sized> {
    type Bean;

    fn get(ctx: &Ctx) -> &Self::Bean;

    fn get_mut(ctx: &mut Ctx) -> &mut Self::Bean;
}

pub mod link {
    use crate::life;

    pub type InitMethod = fn(&mut life::InitPhase);

    #[linkme::distributed_slice]
    pub static INIT_METHODS: [InitMethod] = [..];

    pub type PostInitMethod = fn(&mut life::ActivePhase);

    #[linkme::distributed_slice]
    pub static POST_INIT_METHODS: [PostInitMethod] = [..];

    pub type DropMethod = unsafe fn(&mut life::ActivePhase);

    #[linkme::distributed_slice]
    pub static DROP_METHODS: [DropMethod] = [..];
}

#[derive(Debug)]
pub struct Ctx {
    phase: life::ActivePhase,
}

impl Ctx {
    pub fn new() -> Option<Self> {
        use crate::link::{INIT_METHODS, POST_INIT_METHODS};

        let mut phase = life::InitPhase::take()?;

        for method in INIT_METHODS {
            method(&mut phase);
        }

        let mut phase = unsafe { phase.complete() };

        for method in POST_INIT_METHODS {
            method(&mut phase);
        }

        Some(Ctx { phase })
    }
}

impl Deref for Ctx {
    type Target = life::ActivePhase;

    fn deref(&self) -> &Self::Target {
        &self.phase
    }
}

impl DerefMut for Ctx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.phase
    }
}

impl Drop for Ctx {
    fn drop(&mut self) {
        use crate::link::DROP_METHODS;

        // todo maybe need some sort?
        for method in DROP_METHODS {
            unsafe {
                method(&mut self.phase);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
