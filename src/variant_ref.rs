use vtable::VTable;

use std::any::{Any, TypeId};
use std::fmt::{Debug, Display, Error as FmtError, Formatter};

pub struct VariantRef<'a> {
   pub data: &'a (),
   pub vtable: &'a VTable,
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

impl<'a> Display for VariantRef<'a> {
   fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
      (self.vtable.display)(self, f)
   }
}

impl<'a> Debug for VariantRef<'a> {
   fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
      (self.vtable.debug)(self, f)
   }
}
