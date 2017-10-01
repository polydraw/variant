use vtable::VTable;
use variant_ref::VariantRef;
use variant_ref_mut::VariantRefMut;

use std::any::{Any, TypeId};
use std::fmt::{Debug, Display, Error as FmtError, Formatter};
use std::ops::Deref;


pub struct Variant<'a> {
   pub data: *mut (),
   pub vtable: &'a VTable,
}

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

impl<'a> Deref for Variant<'a> {
   type Target = VariantRef<'a>;

   fn deref(&self) -> &VariantRef<'a> {
      self.as_ref()
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

impl<'a> Clone for Variant<'a> {
   fn clone(&self) -> Self {
      (self.vtable.clone)(self.as_ref())
   }
}

impl<'a> Drop for Variant<'a> {
   fn drop(&mut self) {
      (self.vtable.drop)(self)
   }
}

impl<'a> Display for Variant<'a> {
   fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
      (self.vtable.display)(self.as_ref(), f)
   }
}

impl<'a> Debug for Variant<'a> {
   fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
      (self.vtable.debug)(self.as_ref(), f)
   }
}
