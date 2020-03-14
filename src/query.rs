use legion::prelude::{Read, Write};
use legion::query::{IntoQuery, ViewElement};
use legion::storage::Component;

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
        impl <'a, $($ty: QueryElement),*> Query for ($($ty,)*) {
            type Legion = ($($ty::Legion),*);
        }
    }
}

recursive_macro_call_on_tuple!(impl_query, A, B, C, D, E);
