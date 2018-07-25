//! A convenience macro for creating wrapper enum which may be one of several
//! distinct types. In type theory, this is often referred to as a [sum type].
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
//! # #![cfg_attr(feature = "try_from", feature(try_from))]
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
//! The [`SumType`] trait is also implemented, allowing a basic level of 
//! introspection and dynamic typing.
//!
//! ```rust
//! # #![cfg_attr(feature = "try_from", feature(try_from))]
//! #[macro_use]
//! extern crate sum_type;
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
//! # #![cfg_attr(feature = "try_from", feature(try_from))]
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
//! # Feature Flags
//!
//! The `try_from` feature flag (disabled by default) will implement `TryFrom`
//! to convert a from your sum type back back to one of its variant types.
//!
//! ```rust
//! # #![cfg_attr(feature = "try_from", feature(try_from))]
//! # #![cfg(feature = "try_from")]
//! #[macro_use]
//! extern crate sum_type;
//! # sum_type! { #[derive(Debug, Clone, PartialEq)] pub enum MySumType {
//! #         First(u32), Second(String), Third(Vec<u8>), } }
//! use std::convert::TryFrom;
//!
//! # fn main() {
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
//! [sum type]: https://www.schoolofhaskell.com/school/to-infinity-and-beyond/pick-of-the-week/sum-types
//! [`SumType`]: trait.SumType.html

#![no_std]
#![cfg_attr(feature = "try_from", feature(try_from))]

#[doc(hidden)]
pub extern crate core as _core;

use core::any::Any;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct InvalidType {
    pub expected_variant: &'static str,
    pub actual_variant: &'static str,
}

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

#[cfg(not(feature = "try_from"))]
#[doc(hidden)]
#[macro_export]
macro_rules! __sum_type_try_from { ($($dont_care:tt)*) => ( ) }

#[cfg(feature = "try_from")]
#[doc(hidden)]
#[macro_export]
macro_rules! __sum_type_try_from {
    ($enum_name:ident, $( $name:ident => $type:ty ),*) => {
       $(
            impl $crate::_core::convert::TryFrom<$enum_name> for $type {
                type Error = $crate::InvalidType;

                fn try_from(other: $enum_name) -> Result<$type, Self::Error> {
                    use $crate::SumType;
                    let variant = other.variant();

                    if let $enum_name::$name(value) = other {
                        Ok(value)
                    } else {
                        Err($crate::InvalidType {
                            expected_variant: stringify!($name),
                            actual_variant: variant,
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
    ($enum_name:ident, $( $name:ident => $type:ty ),*) => {
       $(
            impl From<$type> for $enum_name {
                fn from(other: $type) -> $enum_name {
                    $enum_name::$name(other)
                }
            }
        )*
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sum_type_trait {
    ($enum_name:ident, $( $name:ident => $type:ty ),*) => {
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
    ($enum_name:ident, $name:ident => $type:ty) => {
        compile_error!(concat!("The `", stringify!($enum_name), "` type must have more than one variant"));
    };
    ($enum_name:ident, $( $name:ident => $type:ty ),*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __sum_type_impls {
    ($enum_name:ident, $( $name:ident => $type:ty ),*) => (
        __assert_multiple_variants!($enum_name, $( $name => $type ),*);

        __sum_type_from!($enum_name, $($name => $type),*);
        __sum_type_try_from!($enum_name, $($name => $type),*);
        __sum_type_trait!($enum_name, $($name => $type),*);
    )
}

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

        __sum_type_impls!($name, $( $var_name => $var_ty),*);
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

        __sum_type_impls!($name, $( $var_name => $var_ty),*);
    };
}

