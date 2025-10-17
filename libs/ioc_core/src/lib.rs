use std::ops::Deref;

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

pub unsafe trait Registered<K: ?Sized> {
    type Bean;

    fn get(ctx: &Ctx) -> &Self::Bean;
}

#[derive(Debug)]
pub struct Ctx;

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
