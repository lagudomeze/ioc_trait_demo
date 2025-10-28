use std::any::type_name;

pub trait Has<Key> {
    fn get() -> () {
        println!(
            "Got key :{} from module {}",
            type_name::<Key>(),
            type_name::<Self>()
        );
    }
}

pub trait WithModule {
    type Module;
}

pub trait Belong<M> {
    type Parent;
}

impl<K, M, S> Has<K> for M
where
    S: Has<K>,
    S: Belong<M, Parent = M>,
    K: WithModule<Module = S>,
{
    fn get() {
        S::get();
    }
}

macro_rules! contains {
    ($parent:ty, $child:ty) => {
        impl Belong<$parent> for $child {
            type Parent = $parent;
        }

        impl<S> Belong<S> for $child
            where
            $child: Belong<$parent, Parent = $parent>,
            $parent: Belong<S, Parent = S> {
            type Parent = S;
        }
    };
}

macro_rules! impl_has {
    ($module:ty, $key:ty) => {
        impl Has<$key> for $module {}

        impl WithModule for $key {
            type Module = $module;
        }
    };
}

pub struct M0;

pub struct A;

impl_has!(M0, A);

pub struct M1;

pub struct B;

impl_has!(M1, B);

pub struct M2;
pub struct C;

impl_has!(M2, C);

pub struct M3;
pub struct D;

impl_has!(M3, D);

contains!(M1, M0);
// contains!(M2, M0);
contains!(M3, M1);
contains!(M3, M2);


// M3(M1(M0), M2)
// then M3 can access A, B, C, D
// No diamond dependency here
// todo how to support diamond dependency? or not support it at all?

fn main() {
    <M3 as Has<D>>::get();
    <M3 as Has<C>>::get();
    <M3 as Has<B>>::get();
    <M3 as Has<A>>::get();
}
