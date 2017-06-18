use std::any::{TypeId, Any};
use std::clone::Clone;
use std::fmt::{Display, Debug, Formatter, Error as FmtError};

pub struct Variant<'a> {
   data: *mut (),
   vtable: &'a VTable,
}

pub struct VariantRef<'a> {
   data: &'a (),
   vtable: &'a VTable,
}

pub struct VariantRefMut<'a> {
   data: &'a mut (),
   vtable: &'a VTable,
}

pub struct VTable {
   id: TypeId,
   clone: CloneFn,
   drop: DropFn,
   display: DisplayFn,
   debug: DebugFn,
}

type CloneFn = for<'b> fn(&Variant<'b>) -> Variant<'b>;

type DropFn = for<'b> fn(&mut Variant<'b>);

type DisplayFn = for<'b> fn(&Variant<'b>, f: &mut Formatter) -> Result<(), FmtError>;

type DebugFn = DisplayFn;

impl<'a> Variant<'a> {
   pub fn new<T: Any>(value: T, vtable: &'a VTable) -> Self {
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

impl<'a> AsRef<VariantRef<'a>> for Variant<'a> {
   fn as_ref(&self) -> &VariantRef<'a> {
      unsafe { &*(self as *const _ as *const VariantRef<'a>) }
   }
}

impl<'a> AsMut<VariantRefMut<'a>> for Variant<'a> {
   fn as_mut(&mut self) -> &mut VariantRefMut<'a> {
      unsafe { &mut *(self as *mut _ as *mut VariantRefMut<'a>) }
   }
}

impl<'a> VariantRef<'a> {
   pub fn new<T: Any>(value: &'a T, vtable: &'a VTable) -> Self {
      VariantRef {
         data: unsafe { &*(value as *const _ as *const ()) },
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
         unsafe { Some(&*(self.data as *const _ as *const T)) }
      } else {
         None
      }
   }

   #[inline]
   pub unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T {
      debug_assert!(self.is::<T>());

      &*(self.data as *const _ as *const T)
   }
}

impl<'a> VariantRefMut<'a> {
   pub fn new<T: Any>(value: &'a mut T, vtable: &'a VTable) -> Self {
      VariantRefMut {
         data: unsafe { &mut *(value as *mut _ as *mut ()) },
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
         unsafe { Some(&*(self.data as *const _ as *const T)) }
      } else {
         None
      }
   }

   #[inline]
   pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
      if self.is::<T>() {
         unsafe { Some(&mut *(self.data as *mut _ as *mut T)) }
      } else {
         None
      }
   }

   #[inline]
   pub unsafe fn downcast_ref_unchecked<T: Any>(&self) -> &T {
      debug_assert!(self.is::<T>());

      &*(self.data as *const _ as *const T)
   }

   #[inline]
   pub unsafe fn downcast_mut_unchecked<T: Any>(&mut self) -> &mut T {
      debug_assert!(self.is::<T>());

      &mut *(self.data as *mut _ as *mut T)
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

impl VTable {
   pub fn new<T: Any + Clone + Display + Debug>() -> Self {
      VTable {
         id: TypeId::of::<T>(),
         clone: clone_variant::<T>,
         drop: drop_variant::<T>,
         display: display_variant::<T>,
         debug: debug_variant::<T>,
      }
   }

   pub fn variant<T: Any>(&self, value: T) -> Variant {
      Variant::new(value, self)
   }

   pub fn variant_ref<'a, T: Any>(&'a self, value: &'a T) -> VariantRef<'a> {
      VariantRef::new(value, self)
   }

   pub fn variant_ref_mut<'a, T: Any>(&'a self, value: &'a mut T) -> VariantRefMut<'a> {
      VariantRefMut::new(value, self)
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

   #[test]
   fn as_ref() {
      let vtable = VTable::new::<i64>();
      let variant = vtable.variant(1234_i64);

      fn inner(variant: &VariantRef) {
         assert_eq!(Some(&1234_i64), variant.downcast_ref::<i64>());
      }

      inner(variant.as_ref());
   }

   #[test]
   fn as_mut() {
      let vtable = VTable::new::<i64>();
      let mut variant = vtable.variant(1234_i64);

      fn inner(variant: &mut VariantRefMut) {
         *variant.downcast_mut::<i64>().unwrap() *= 2;
      }

      inner(variant.as_mut());

      assert_eq!(Some(&2468_i64), variant.downcast_ref::<i64>());
   }

   #[test]
   fn stack_ref() {
      let vtable = VTable::new::<i64>();
      let values: [i64; 4] = [0, 2, 5, 6];
      let variants = [
         vtable.variant_ref(&values[0]),
         vtable.variant_ref(&values[1]),
         vtable.variant_ref(&values[2]),
         vtable.variant_ref(&values[3]),
      ];
      let mut sum: i64 = 0;
      for variant in &variants {
         sum += *variant.downcast_ref::<i64>().unwrap();
      }
      assert_eq!(13_i64, sum);
   }
}
