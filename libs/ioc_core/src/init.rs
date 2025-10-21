use crate::config::{CfgSource, ConfigSource, IsConfig};
use crate::life::InitPhase;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct InitCtx {
    phase: InitPhase,
    config: CfgSource,
}

impl InitCtx {
    pub fn new(phase: InitPhase, config: CfgSource) -> Self {
        Self { phase, config }
    }

    pub fn into_phase(self) -> InitPhase {
        self.phase
    }
}

impl ConfigSource for InitCtx {
    fn get_config<T: IsConfig>(&self, key: impl AsRef<str>) -> crate::Result<T> {
        self.config.get_config(key)
    }
    fn get_config_or<T: IsConfig>(&self, key: impl AsRef<str>, default: T) -> crate::Result<T> {
        self.config.get_config_or(key, default)
    }
}

impl Deref for InitCtx {
    type Target = InitPhase;

    fn deref(&self) -> &Self::Target {
        &self.phase
    }
}

impl DerefMut for InitCtx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.phase
    }
}
