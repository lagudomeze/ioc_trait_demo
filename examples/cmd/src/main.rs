use crate::mod2::AliasHaha;
use a::{A, AKey, SomeNeedA};
use ioc::Bean;
use ioc::prelude::*;

pub mod module {
    use crate::Ctx as Root;
    use std::ops::Deref;

    #[derive(Debug)]
    pub struct Ctx(Root);

    impl Deref for Ctx {
        type Target = Root;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    pub fn new(root: Root) -> Ctx {
        Ctx(root)
    }
}

#[derive(Debug, Bean)]
pub struct AImplByMain;

impl A for AImplByMain {
    fn test(&self) {
        println!("AImplByMain");
    }
}

impl Alias<AKey> for module::Ctx {
    type Key = AImplByMain;
}

pub mod mod2 {
    use crate::{Alias, Ctx as Root};
    use a::SomeNeedA;
    use std::ops::Deref;

    pub struct Mod2(pub Root);

    impl Deref for Mod2 {
        type Target = Root;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    pub struct Impl<T>(T);

    type SrcMod = crate::module::Ctx;
    type SrcModKey<N> = <SrcMod as Alias<N>>::Key;

    impl<N> Alias<N> for Mod2
    where
        SrcMod: Alias<N>,
    {
        type Key = SrcModKey<N>;
    }

    pub struct AliasHaha;

    impl Alias<AliasHaha> for Mod2 {
        type Key = SomeNeedA;
    }
}

fn main() {
    use crate::Ctx as Root;
    let ctx = Root::new().unwrap();

    // let _ctx = module::new(ctx);

    let ctx = mod2::Mod2(ctx);

    let some_need_a = ctx.get_by_key::<AImplByMain>();

    some_need_a.test();

    let some_need_a = ctx.get_by_alias::<AKey>();

    some_need_a.test();

    let some_need_a = ctx.get_by_alias::<AliasHaha>();

    some_need_a.test(&ctx);

    let some_need_a = ctx.get_by_key::<SomeNeedA>();

    some_need_a.test(&ctx);
    some_need_a.test2(&ctx);
}
