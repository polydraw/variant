#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

use std::any::{TypeId, Any};
use std::clone::Clone;
use std::fmt::{Display, Debug, Formatter, Error as FmtError};

pub struct Variant<'a> {
   data: *mut (),
   vtable: &'a VTable<'a>,
}

impl<'a> Variant<'a> {
   pub fn new<T: Any>(value: T, vtable: &'a VTable<'a>) -> Self {
      Variant {
         data: Box::into_raw(Box::new(value)) as *mut (),
         vtable: vtable,
      }
   }

   #[inline]
   pub fn is<T: Any>(&self) -> bool {
      self.vtable.id == TypeId::of::<T>()
   }

   #[inline]
   pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
      if self.is::<T>() {
         unsafe { Some(&*(self.data as *const T)) }
      } else {
         None
      }
   }

   #[inline]
   pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
      if self.is::<T>() {
         unsafe { Some(&mut *(self.data as *mut T)) }
      } else {
         None
      }
   }

   #[inline]
   pub unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T {
      debug_assert!(self.is::<T>());

      &*(self.data as *const T)
   }

   #[inline]
   pub unsafe fn downcast_mut_unchecked<T: Any>(&mut self) -> &mut T {
      debug_assert!(self.is::<T>());

      &mut *(self.data as *mut T)
   }
}

impl<'a> Clone for Variant<'a> {
   fn clone(&self) -> Self {
      (self.vtable.clone)(self)
   }
}

impl<'a> Drop for Variant<'a> {
   fn drop(&mut self) {
      (self.vtable.drop)(self)
   }
}

impl<'a> Display for Variant<'a> {
   fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
      (self.vtable.display)(self, f)
   }
}

impl<'a> Debug for Variant<'a> {
   fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
      (self.vtable.debug)(self, f)
   }
}

type CloneFn<'a> = fn(&Variant<'a>) -> Variant<'a>;

type DropFn<'a> = fn(&mut Variant<'a>);

type DisplayFn<'a> = fn(&Variant<'a>, f: &mut Formatter) -> Result<(), FmtError>;

type DebugFn<'a> = DisplayFn<'a>;

pub struct VTable<'a> {
   id: TypeId,
   clone: CloneFn<'a>,
   drop: DropFn<'a>,
   display: DisplayFn<'a>,
   debug: DebugFn<'a>,
}

impl<'a> VTable<'a> {
   #[cfg_attr(feature = "cargo-clippy", allow(new_without_default))]
   pub fn new<T: Any + Clone + Display + Debug>() -> Self {
      VTable {
         id: TypeId::of::<T>(),
         clone: clone_variant::<T>,
         drop: drop_variant::<T>,
         display: display_variant::<T>,
         debug: debug_variant::<T>,
      }
   }

   pub fn variant<T: Any>(&'a self, value: T) -> Variant<'a> {
      Variant::new(value, self)
   }
}

fn clone_variant<'a, T: Any + Clone>(variant: &Variant<'a>) -> Variant<'a> {
   Variant::new(unsafe { variant.downcast_ref_unchecked() as &T }.clone(), variant.vtable)
}

fn drop_variant<T>(variant: &mut Variant) {
   drop(unsafe { *Box::from_raw(variant.data as *mut T) });
}

fn display_variant<T: Any + Display>(variant: &Variant, f: &mut Formatter) -> Result<(), FmtError> {
   Display::fmt(unsafe { variant.downcast_ref_unchecked() as &T }, f)
}

fn debug_variant<T: Any + Debug>(variant: &Variant, f: &mut Formatter) -> Result<(), FmtError> {
   Debug::fmt(unsafe { variant.downcast_ref_unchecked() as &T }, f)
}

#[cfg(test)]
mod tests {
   use std::fmt::{Display, Debug};

   use super::*;

   #[test]
   fn clone() {
      let vtable = VTable::new::<i64>();
      let origin = vtable.variant(1234_i64);
      let cloned = origin.clone();
      assert_eq!(origin.downcast_ref::<i64>(), cloned.downcast_ref::<i64>());
      assert_eq!(Some(&1234_i64), cloned.downcast_ref::<i64>());
   }

   #[test]
   fn drop_count() {
      #[derive(Clone, Debug)]
      struct Dropper(*mut usize);

      impl Drop for Dropper {
         fn drop(&mut self) {
            unsafe { *self.0 += 1 };
         }
      }

      impl Display for Dropper {
         fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
            Debug::fmt(self, f)
         }
      }

      let vtable = VTable::new::<Dropper>();
      let mut dropped: usize = 0;
      let variant = vtable.variant(Dropper(&mut dropped));
      drop(variant);
      assert_eq!(dropped, 1);

      dropped = 0;
      let variant = vtable.variant(Dropper(&mut dropped));
      let clone = variant.clone();
      drop(variant);
      drop(clone);
      assert_eq!(dropped, 2);
   }

   #[test]
   fn debug() {
      let vtable = VTable::new::<&str>();
      let variant = vtable.variant("data");
      assert_eq!("\"data\"", format!("{:?}", variant));

      let vtable = VTable::new::<i64>();
      let variant = vtable.variant(1234_i64);
      assert_eq!("1234", format!("{:?}", variant));
   }

   #[test]
   fn display() {
      let vtable = VTable::new::<&str>();
      let variant = vtable.variant("data");
      assert_eq!("data", format!("{}", variant));

      let vtable = VTable::new::<i64>();
      let variant = vtable.variant(1234_i64);
      assert_eq!("1234", format!("{}", variant));
   }
}
