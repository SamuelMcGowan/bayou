macro_rules! index_types {
    ($($t:ident),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $t(pub usize);
        )*
    };
}
pub(crate) use index_types;
