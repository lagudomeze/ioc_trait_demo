use crate::config::CfgParams;
use std::ops::{Deref, DerefMut};

pub mod config;
pub mod error;
pub mod init;
pub mod life;
pub mod place;

pub type Result<T> = std::result::Result<T, error::Error>;

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

/// # Safety
/// This trait is unsafe because incorrect implementation may lead to undefined behavior.
/// Must need add link section to register the bean.
/// see [linkme](https://crates.io/crates/linkme) and [link mod](link) for more details.
pub unsafe trait Registered<K: ?Sized> {
    type Bean;

    fn get(ctx: &Ctx) -> &Self::Bean;

    fn get_mut(ctx: &mut Ctx) -> &mut Self::Bean;
}

pub mod link {
    use crate::life;

    pub type InitMethod = fn(ctx: &mut crate::init::InitCtx) -> crate::Result<()>;

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
    pub fn new() -> Result<Self> {
        Self::from_cfg(CfgParams::default())
    }

    pub fn from_cfg(param: CfgParams) -> Result<Self> {
        use crate::config::CfgSource;
        use crate::link::{INIT_METHODS, POST_INIT_METHODS};

        let phase = life::InitPhase::take()?;
        let cfg_source = CfgSource::new(param)?;
        let mut ctx = init::InitCtx::new(phase, cfg_source);

        for method in INIT_METHODS {
            method(&mut ctx)?;
        }
        let phase = ctx.into_phase();

        let mut phase = unsafe { phase.complete() };

        for method in POST_INIT_METHODS {
            method(&mut phase);
        }

        Ok(Ctx { phase })
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
