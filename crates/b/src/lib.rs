use ioc::Bean;
use ioc::prelude::*;


#[derive(Debug, Bean)]
pub struct B;

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