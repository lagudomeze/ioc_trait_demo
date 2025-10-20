use b::B;
use ioc::Bean;
use ioc::prelude::*;

#[allow(non_camel_case_types)]
pub struct _trait_a_key_;

pub trait A {
    fn test(&self);
}

pub trait HasA: Alias<_trait_a_key_>
where
    Ctx: Registered<<Self as Alias<_trait_a_key_>>::Key, Bean: A>,
{
}

impl<C> HasA for C
where
    C: Alias<_trait_a_key_>,
    Ctx: Registered<<C as Alias<_trait_a_key_>>::Key, Bean: A>,
{
}

#[derive(Debug, Bean)]
pub struct SomeNeedA;

impl SomeNeedA {
    pub fn test2<C>(&self, ctx: &C)
    where
        C: Context,
        Ctx: Registered<B, Bean = B>,
        C: Alias<_trait_a_key_>,
        Ctx: Registered<<C as Alias<_trait_a_key_>>::Key, Bean: A>,
    {
        let b = ctx.get_by_key::<B>();
        println!("{}", b.test());
        self.test(ctx);
    }

    pub fn test<C>(&self, ctx: &C)
    where
        C: Context,
        Ctx: Registered<B, Bean = B>,
        C: Alias<_trait_a_key_>,
        Ctx: Registered<<C as Alias<_trait_a_key_>>::Key, Bean: A>,
        //todo: Ctx: Registered<SomeNeedA, Bean = SomeNeedA>,
    {
        let a = ctx.get_by_alias::<_trait_a_key_>();
        a.test();
        let b = ctx.get_by_key::<B>();
        println!("{}", b.test());
        b.test2(&ctx);
    }
}
