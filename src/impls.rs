macro_rules! impl_partial_eq {
    ($lhs:ty, $rhs:ty) => {
        impl<'a, const MAX: usize, E> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                let this: &str = self.as_ref();
                let other: &str = other.as_ref();
                PartialEq::eq(this, other)
            }
        }

        impl<'a, const MAX: usize, E> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                let this: &str = self.as_ref();
                let other: &str = other.as_ref();
                PartialEq::eq(this, other)
            }
        }
    };
}

mod bstr {
    use crate::bstr::BStr;

    impl_partial_eq!(BStr<MAX, E>, str);
    impl_partial_eq!(BStr<MAX, E>, String);
    impl_partial_eq!(&'a BStr<MAX, E>, String);
}

mod bstring {
    use crate::bstr::BStr;
    use crate::bstring::BString;

    impl_partial_eq!(BString<MAX, E>, str);
    impl_partial_eq!(BString<MAX, E>, &'a str);
    impl_partial_eq!(BString<MAX, E>, String);
    impl_partial_eq!(BString<MAX, E>, BStr<MAX, E>);
    impl_partial_eq!(BString<MAX, E>, &'a BStr<MAX, E>);
}
