use ioc_core::{Ctx, Registered};

pub struct B;

unsafe impl Registered<B> for Ctx {
    type Bean = B;

    #[inline(always)]
    fn get(_: &Ctx) -> &Self::Bean {
        &B
    }
}

impl B {
    pub fn test(&self) -> &'static str {
        println!("test b");
        "hello this is b"
    }
    pub fn test2(&self, ctx: &Ctx) {
        println!("test b with {ctx:?}");
    }
}
