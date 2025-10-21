pub mod prelude {
    pub use ::ioc_core::{
        Alias, Context, Ctx, Registered, Result, config::*, error::Error, init::*, life::*,
        link::*, place::*,
    };
}

pub use prelude::Result;

pub use ioc_macros::*;
