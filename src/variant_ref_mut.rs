use vtable::VTable;
use variant_ref::VariantRef;

use std::any::{TypeId, Any};
use std::fmt::{Display, Debug, Formatter, Error as FmtError};

pub struct VariantRefMut<'a> {
   data: &'a mut (),
   vtable: &'a VTable,
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

impl<'a> AsRef<VariantRef<'a>> for VariantRefMut<'a> {
   fn as_ref(&self) -> &VariantRef<'a> {
      unsafe { &*(self as *const _ as *const VariantRef<'a>) }
   }
}

impl<'a> Display for VariantRefMut<'a> {
   fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
      (self.vtable.display)(self.as_ref(), f)
   }
}

impl<'a> Debug for VariantRefMut<'a> {
   fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
      (self.vtable.debug)(self.as_ref(), f)
   }
}
