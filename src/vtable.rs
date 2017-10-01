use variant::Variant;
use variant_ref::VariantRef;
use variant_ref_mut::VariantRefMut;

use std::any::{Any, TypeId};
use std::fmt::{Debug, Display, Error as FmtError, Formatter};

type CloneFn = for<'b> fn(&VariantRef<'b>) -> Variant<'b>;

type DropFn = for<'b> fn(&mut Variant<'b>);

type DisplayFn = for<'b> fn(&VariantRef<'b>, f: &mut Formatter)
   -> Result<(), FmtError>;

type DebugFn = DisplayFn;

pub struct VTable {
   pub id: TypeId,
   pub clone: CloneFn,
   pub drop: DropFn,
   pub display: DisplayFn,
   pub debug: DebugFn,
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

fn clone_variant<'a, T: Any + Clone>(variant: &VariantRef<'a>) -> Variant<'a> {
   Variant::new(unsafe { variant.downcast_ref_unchecked() as &T }.clone(), variant.vtable)
}

fn drop_variant<T>(variant: &mut Variant) {
   drop(unsafe { *Box::from_raw(variant.data as *mut _ as *mut T) });
}

fn display_variant<T: Any + Display>(
   variant: &VariantRef,
   f: &mut Formatter,
) -> Result<(), FmtError> {
   Display::fmt(unsafe { variant.downcast_ref_unchecked() as &T }, f)
}

fn debug_variant<T: Any + Debug>(variant: &VariantRef, f: &mut Formatter) -> Result<(), FmtError> {
   Debug::fmt(unsafe { variant.downcast_ref_unchecked() as &T }, f)
}
