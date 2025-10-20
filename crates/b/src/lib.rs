use linkme::distributed_slice;
use ioc::prelude::*;


pub struct B;

static B_HOLDER: StaticPlace<B> = StaticPlace::uninit();

#[distributed_slice(INIT_METHODS)]
static B_INIT_METHOD: InitMethod = init_b;

fn init_b(phase: &mut InitPhase) {
    B_HOLDER.initialize(phase).write(B);
}

#[distributed_slice(DROP_METHODS)]
static B_DROP_METHOD: DropMethod = drop_b;

#[inline]
fn drop_b(phase: &mut ActivePhase) {
    println!("dropping B");
    unsafe {
        B_HOLDER.deinitialize(phase)
    }
}

unsafe impl Registered<B> for Ctx {
    type Bean = B;

    #[inline(always)]
    fn get(ctx: &Ctx) -> &Self::Bean {
        B_HOLDER.get(ctx)
    }

    fn get_mut(ctx: &mut Ctx) -> &mut Self::Bean {
        B_HOLDER.get_mut(ctx)
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b() {
        let ctx = Ctx::new().unwrap();
        let x = &ctx;

        let b = x.get_by_key::<B>();
        assert_eq!(b.test(), "hello this is b");
        b.test2(x);
    }}