extern crate core;

use crate::mod2::AliasHaha;
use a::{_trait_a_key_, A, SomeNeedA};
use ioc_core::{self, Alias, Context, Ctx as Root, Registered};

pub mod module {
    use ioc_core::Ctx as Root;
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

pub struct AImplByMain;

impl A for AImplByMain {
    fn test(&self) {
        println!("AImplByMain");
    }
}

unsafe impl Registered<AImplByMain> for Root {
    type Bean = AImplByMain;

    #[inline(always)]
    fn get(_: &Root) -> &Self::Bean {
        &AImplByMain
    }
}

impl Alias<_trait_a_key_> for module::Ctx {
    type Key = AImplByMain;
}

pub mod mod2 {
    use a::SomeNeedA;
    use ioc_core::{Alias, Ctx as Root};
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
    let _ctx = module::new(Root);

    let ctx = mod2::Mod2(Root);

    let some_need_a = ctx.get_by_key::<AImplByMain>();

    some_need_a.test();

    let some_need_a = ctx.get_by_alias::<_trait_a_key_>();

    some_need_a.test();

    let some_need_a = ctx.get_by_alias::<AliasHaha>();

    some_need_a.test(&ctx);

    let some_need_a = ctx.get_by_key::<SomeNeedA>();

    some_need_a.test(&ctx);
    some_need_a.test2(&ctx);
}
