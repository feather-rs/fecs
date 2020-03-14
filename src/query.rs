use crate::World;
use legion::prelude::{Entity, Read, Write};
use legion::query::View;
use legion::query::{IntoQuery, ViewElement};
use legion::storage::Component;

pub struct QueryBorrow<'a, Q>
where
    Q: Query,
{
    pub(crate) world: &'a mut World,
    pub(crate) inner:
        legion::query::Query<Q::Legion, <Q::Legion as legion::query::DefaultFilter>::Filter>,
}

impl<'a, Q> QueryBorrow<'a, Q>
where
    Q: Query,
{
    pub fn iter(&mut self) -> impl Iterator<Item = <<Q::Legion as View>::Iter as Iterator>::Item> {
        self.inner.iter_mut(self.world.inner_mut())
    }

    pub fn iter_entities(
        &mut self,
    ) -> impl Iterator<Item = (Entity, <<Q::Legion as View>::Iter as Iterator>::Item)> {
        self.inner.iter_entities_mut(self.world.inner_mut())
    }
}

pub trait Query {
    type Legion: IntoQuery;
}

pub trait QueryElement {
    type Legion: IntoQuery + ViewElement;
}

impl QueryElement for () {
    type Legion = Read<()>;
}

impl<'a, T> QueryElement for &'a T
where
    T: Component,
{
    type Legion = Read<T>;
}

impl<'a, T> QueryElement for &'a mut T
where
    T: Component,
{
    type Legion = Write<T>;
}

macro_rules! recursive_macro_call_on_tuple {
    ($m: ident, $ty: ident) => {
        $m!{$ty}
    };
    ($m: ident, $ty: ident, $($tt: ident),*) => {
        $m!{$ty, $($tt),*}
        recursive_macro_call_on_tuple!{$m, $($tt),*}
    };
}

macro_rules! impl_query {
    ($($ty:ident),+) => {
        #[allow(unused_parens)]
        impl <'a, $($ty: QueryElement,)*> Query for ($($ty),*) {
            type Legion = ($($ty::Legion),*);
        }
    }
}

recursive_macro_call_on_tuple!(impl_query, A, B, C, D, E);
