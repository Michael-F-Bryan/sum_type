//! A convenience macro for creating a wrapper enum which may be one of several
//! distinct types. In type theory, this is often referred to as a [sum type].
//!
//! This crate will work with `no_std` code.
//!
//! # Examples
//!
//! Using the `sum_type!()` macro is rather straightforward. You just define a
//! normal `enum` inside it and the macro will automatically add a bunch of
//! handy trait implementations.
//!
//! For convenience, all attributes are passed through and the macro will
//! derive `From` for each variant.
//!
//! ```rust
//! #[macro_use]
//! extern crate sum_type;
//!
//! sum_type! {
//!     #[derive(Debug, Clone, PartialEq)]
//!     pub enum MySumType {
//!         /// The first variant.
//!         First(u32),
//!         /// The second variant.
//!         Second(String),
//!         /// A list of bytes.
//!         Third(Vec<u8>),
//!     }
//! }
//!
//! # fn main() {
//! let first: MySumType = 52.into();
//! assert_eq!(first, MySumType::First(52));
//! # }
//! ```
//!
//! You can also be lazy and omit the variant name. This will name the variant
//! the same thing as its type.
//!
//! ```rust
//! # #[macro_use]
//! # extern crate sum_type;
//! sum_type!{
//!     pub enum Lazy {
//!         f32, u32, String,
//!     }
//! }
//! # fn main() {
//! let s = Lazy::String("Hello World!".to_string());
//! # }
//! ```
//!
//! The [`SumType`] trait is also implemented, allowing a basic level of
//! introspection and dynamic typing.
//!
//! ```rust
//! # #[macro_use]
//! # extern crate sum_type;
//! use sum_type::SumType;
//! # sum_type! { #[derive(Debug, Clone, PartialEq)] pub enum MySumType {
//! #         First(u32), Second(String), Third(Vec<u8>), } }
//!
//! # fn main() {
//! let first = MySumType::First(52);
//!
//! assert_eq!(first.variant(), "First");
//! assert_eq!(first.variants(), &["First", "Second", "Third"]);
//! assert!(first.variant_is::<u32>());
//! assert_eq!(first.downcast_ref::<u32>(), Some(&52));
//! # }
//! ```
//!
//! # Assumptions
//!
//! You need to make sure your type has more than one variant, meaning the
//! following example will fail to compile.
//!
//! ```rust,compile_fail
//! # fn main() {}
//! #[macro_use]
//! extern crate sum_type;
//!
//! sum_type!{
//!     pub enum OneVariant {
//!         First(String),
//!     }
//! }
//! ```
//!
//! The `compile_error!()` macro is used to give a (hopefully) useful error
//! message.
//!
//! ```text
//! error: The `OneVariant` type must have more than one variant
//!   --> src/lib.rs:37:1
//!    |
//! 7  | / sum_type!{
//! 8  | |     pub enum OneVariant {
//! 9  | |         First(String),
//! 10 | |     }
//! 11 | | }
//!    | |_^
//!    |
//!    = note: this error originates in a macro outside of the current crate
//! ```
//!
//! Sum types containing generics, including lifetimes, or which are using
//! visibility modifiers (e.g. `pub(crate)`) aren't (yet!) supported. That
//! means this will fail:
//!
//! ```rust,compile_fail
//! # fn main() {}
//! # #[macro_use]
//! # extern crate sum_type;
//! sum_type!{
//!     TypeWithLifetime<'a> {
//!         First(&'a str),
//!         Second(usize),
//!     }
//! }
//! ```
//!
//! And so will this:
//!
//! ```rust,compile_fail
//! # fn main() {}
//! # #[macro_use]
//! # extern crate sum_type;
//! sum_type!{
//!     pub(crate) ModifiedVisibility {
//!         First(u32),
//!         Second(String),
//!     }
//! }
//! ```
//!
//! # Try From
//!
//! `TryFrom` is automatically implemented on your sum type to convert it back to one of its variant types.
//!
//! ```rust
//! #[macro_use]
//! extern crate sum_type;
//! # fn main() {
//! # sum_type! { #[derive(Debug, Clone, PartialEq)] pub enum MySumType {
//! #         First(u32), Second(String), Third(Vec<u8>), } }
//! use std::convert::TryFrom;
//!
//! let first = MySumType::First(52);
//!
//! let as_u32 = u32::try_from(first);
//! assert_eq!(as_u32, Ok(52));
//!
//! let second = MySumType::Second(String::from("Not a Vec<u8>"));
//! let as_vec_u8 = Vec::<u8>::try_from(second);
//! assert!(as_vec_u8.is_err());
//!
//! let err = as_vec_u8.unwrap_err();
//! assert_eq!(err.expected_variant, "Third");
//! assert_eq!(err.actual_variant, "Second");
//! # }
//! ```
//!
//! The `generated_example` feature flag will create an example of our
//! `MySumType` which can be viewed using `rustdoc`.
//!
//! [sum type]: https://www.schoolofhaskell.com/school/to-infinity-and-beyond/pick-of-the-week/sum-types
//! [`SumType`]: trait.SumType.html

#![no_std]
#![deny(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations,
    unsafe_code
)]

// re-export so users of our macro have a stable way to import the standard
// library (as `$crate::_core`).
#[doc(hidden)]
pub extern crate core as _core;

use core::any::Any;

/// The result of a failed conversion from `TryFrom`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct InvalidType {
    /// The variant this conversion is valid for.
    pub expected_variant: &'static str,
    /// The actual variant.
    pub actual_variant: &'static str,
    /// All possible variants.
    pub all_variants: &'static [&'static str],
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

/// Various methods for introspection and dynamic typing.
///
/// # Note
///
/// This trait is automatically implemented for all types generated by the
/// `sum_type!()` macro. You should never need to implement it manually.
pub trait SumType {
    /// The name of the current variant.
    fn variant(&self) -> &'static str;
    /// A list of all possible variants.
    fn variants(&self) -> &'static [&'static str];
    /// Try to get a reference to the inner field if it is a `T`.
    fn downcast_ref<T: Any>(&self) -> Option<&T>;
    /// Return a mutable reference to the inner field if it is a `T`.
    fn downcast_mut<T: Any>(&mut self) -> Option<&mut T>;
    /// Is the underlying variant an instance of `T`?
    fn variant_is<T: Any>(&self) -> bool;
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sum_type_try_from {
    ($enum_name:ident, $( $name:ident => $variant_type:ty ),*) => {
       $(
            impl $crate::_core::convert::TryFrom<$enum_name> for $variant_type {
                type Error = $crate::InvalidType;

                fn try_from(other: $enum_name) -> Result<$variant_type, Self::Error> {
                    let variant = $crate::SumType::variant(&other);
                    let variants = $crate::SumType::variants(&other);

                    if let $enum_name::$name(value) = other {
                        Ok(value)
                    } else {
                        Err($crate::InvalidType {
                            expected_variant: stringify!($name),
                            actual_variant: variant,
                            all_variants: variants,
                            __non_exhaustive: (),
                        })
                    }
                }

            }
       )*
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sum_type_from {
    ($enum_name:ident, $( $name:ident => $variant_type:ty ),*) => {
       $(
            impl From<$variant_type> for $enum_name {
                fn from(other: $variant_type) -> $enum_name {
                    $enum_name::$name(other)
                }
            }
        )*
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sum_type_trait {
    ($enum_name:ident, $( $name:ident => $variant_type:ty ),*) => {
        impl $crate::SumType for $enum_name {
            fn variants(&self) -> &'static [ &'static str] {
                &[
                    $( stringify!($name) ),*
                ]
            }

            fn variant(&self) ->  &'static str {
                match *self {
                    $(
                        $enum_name::$name(_) => stringify!($name),
                    )*
                }
            }

            fn downcast_ref<T: $crate::_core::any::Any>(&self) -> Option<&T> {
                use $crate::_core::any::Any;

                match *self {
                    $(
                        $enum_name::$name(ref value) => (value as &Any).downcast_ref::<T>(),
                    )*
                }
            }

            fn downcast_mut<T: $crate::_core::any::Any>(&mut self) -> Option<&mut T> {
                use $crate::_core::any::Any;

                match *self {
                    $(
                        $enum_name::$name(ref mut value) => (value as &mut Any).downcast_mut::<T>(),
                    )*
                }
            }

            fn variant_is<T: $crate::_core::any::Any>(&self) -> bool {
                self.downcast_ref::<T>().is_some()
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __assert_multiple_variants {
    ($enum_name:ident, $name:ident => $variant_type:ty) => {
        compile_error!(concat!(
            "The `",
            stringify!($enum_name),
            "` type must have more than one variant"
        ));
    };
    ($enum_name:ident, $( $name:ident => $variant_type:ty ),*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sum_type_impls {
    ($enum_name:ident, $( $name:ident => $variant_type:ty ),*) => (
        $crate::__assert_multiple_variants!($enum_name, $( $name => $variant_type ),*);

        $crate::__sum_type_from!($enum_name, $($name => $variant_type),*);
        $crate::__sum_type_try_from!($enum_name, $($name => $variant_type),*);
        $crate::__sum_type_trait!($enum_name, $($name => $variant_type),*);
    )
}

/// The entire point.
#[macro_export]
macro_rules! sum_type {
    (
        $( #[$outer:meta] )*
        pub enum $name:ident {
            $(
                $( #[$inner:meta] )*
                $var_name:ident($var_ty:ty),
                )*
        }) => {
       $( #[$outer] )*
        pub enum $name {
            $(
                $( #[$inner] )*
                $var_name($var_ty),
            )*
        }

        $crate::__sum_type_impls!($name, $( $var_name => $var_ty),*);
    };
    (
        $( #[$outer:meta] )*
        enum $name:ident {
            $(
                $( #[$inner:meta] )*
                $var_name:ident($var_ty:ty),
                )*
        }) => {
       $( #[$outer] )*
        enum $name {
            $(
                $( #[$inner] )*
                $var_name($var_ty),
            )*
        }

        $crate::__sum_type_impls!($name, $( $var_name => $var_ty),*);
    };

    // "lazy" variations which reuse give the variant the same name as its type.
    (
        $( #[$outer:meta] )*
        pub enum $name:ident {
            $(
                $( #[$inner:meta] )*
                $var_name:ident,
                )*
        }) => {
            $crate::sum_type!($(#[$outer])* pub enum $name { $( $(#[$inner])* $var_name($var_name), )* });
    };
    (
        $( #[$outer:meta] )*
        enum $name:ident {
            $(
                $( #[$inner:meta] )*
                $var_name:ident($var_ty:ty),
                )*
        }) => {
            $crate::sum_type!($(#[$outer])* enum $name { $( $(#[$inner])* $var_name($var_name), )* });
    };
}

/// Execute an operation on each enum variant.
///
/// This macro is short-hand for matching on each variant in an enum and
/// performing the same operation to each.
///
/// It will expand to roughly the following:
///
/// ```rust
/// sum_type::sum_type! {
///     #[derive(Debug, PartialEq)]
///     pub enum Foo {
///         First(u32),
///         Second(f64),
///         Third(String),
///     }
/// }
///
/// let third = Foo::Third(String::from("Hello World"));
///
/// let got = match third {
///     Foo::First(ref item) => item.to_string(),
///     Foo::Second(ref item) => item.to_string(),
///     Foo::Third(ref item) => item.to_string(),
/// };
/// ```
///
/// # Examples
///
/// ```rust
/// sum_type::sum_type! {
///     #[derive(Debug, PartialEq)]
///     pub enum Foo {
///         First(u32),
///         Second(f64),
///         Third(String),
///     }
/// }
///
/// let mut third = Foo::Third(String::from("Hello World"));
///
/// // Execute some operation on each variant (skipping Second) and get the
/// // return value
/// let mut got = sum_type::defer!(Foo as third; First | Third => |ref item| item.to_string());
///
/// assert_eq!(got, "Hello World");
///
/// // mutate the variant in place
/// sum_type::defer!(Foo as third;
///     First | Second | Third => |ref mut item| {
///         *item = Default::default();
///     }
/// );
/// assert_eq!(third, Foo::Third(String::new()));
/// ```
///
/// The `defer!()` macro will panic if it encounters an unhandled variant.
///
/// ```rust,should_panic
/// sum_type::sum_type! {
///     #[derive(Debug, PartialEq)]
///     pub enum Foo {
///         First(u32),
///         Second(f64),
///         Third(String),
///     }
/// }
///
/// let mut first = Foo::First(42);
///
/// sum_type::defer!(Foo as first; Second | Third => |ref _dont_care| ());
/// ```
#[macro_export]
macro_rules! defer {
    ($kind:ident as $variable:expr; $( $variant:ident )|* => |ref $item:ident| $exec:expr) => {
        $crate::defer!(@foreach_variant $kind, $variable;
            $(
                $kind::$variant(ref $item) => $exec
            ),*
        )
    };
    ($kind:ident as $variable:expr; $( $variant:ident )|* => |ref mut $item:ident| $exec:expr) => {
        $crate::defer!(@foreach_variant $kind, $variable;
            $(
                $kind::$variant(ref mut $item) => $exec
            ),*
        )
    };
    (@foreach_variant $kind:ident, $variable:expr; $( $pattern:pat => $exec:expr ),*) => {
        match $variable {
            $(
                $pattern => $exec,
            )*
            #[allow(unreachable_patterns)]
            _ => unreachable!("Unexpected variant, {}, for {}",
                <_ as $crate::SumType>::variant(&$variable),
                stringify!($kind)),
        }
    }
}

/// An example of the generated sum type.
#[cfg(feature = "generated_example")]
#[allow(missing_docs)]
pub mod generated_example {
    sum_type! {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum MySumType {
            /// The first variant.
            First(u32),
            /// The second variant.
            Second(&'static str),
            /// A list of bytes.
            Third(&'static [u8]),
        }
    }
}
