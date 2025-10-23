use b::B;
use ioc::Bean;
use ioc::prelude::*;
use ioc::with;

pub trait A {
    fn test(&self);
}

pub struct AKey;

#[derive(Debug, Bean)]
pub struct SomeNeedA;

impl SomeNeedA {
    #[with(bean(path = B))]
    #[with(alias(name = AKey, traits = A))]
    pub fn test2<C>(&self, ctx: &C)
    where
        C: Context,
    {
        let b = ctx.get_by_key::<B>();
        println!("{}", b.test());
        self.test(ctx);
    }

    pub fn test<Cxx>(&self, ctx: &Cxx)
    where
        Cxx: Context,
        Ctx: Registered<B, Bean = B>,
        Cxx: Alias<AKey>,
        Ctx: Registered<<Cxx as Alias<AKey>>::Key, Bean: A>
    {
        let a = ctx.get_by_alias::<AKey>();
        a.test();
        let b = ctx.get_by_key::<B>();
        println!("{}", b.test());
        b.test2(ctx);
    }
}
