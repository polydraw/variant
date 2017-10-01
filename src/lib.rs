mod vtable;
mod variant;
mod variant_ref;
mod variant_ref_mut;

pub use vtable::VTable;
pub use variant::Variant;
pub use variant_ref::VariantRef;
pub use variant_ref_mut::VariantRefMut;


#[cfg(test)]
mod tests {
   use std::fmt::{Debug, Display, Error as FmtError, Formatter};

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
   fn debug_ref() {
      let vtable = VTable::new::<&str>();
      let data = "data";
      let variant_ref = vtable.variant_ref(&data);
      assert_eq!("\"data\"", format!("{:?}", variant_ref));

      let vtable = VTable::new::<i64>();
      let data = 1234_i64;
      let variant_ref = vtable.variant_ref(&data);
      assert_eq!("1234", format!("{:?}", variant_ref));
   }

   #[test]
   fn debug_ref_mut() {
      let vtable = VTable::new::<&str>();
      let mut data = "data";
      let variant_ref = vtable.variant_ref_mut(&mut data);
      assert_eq!("\"data\"", format!("{:?}", variant_ref));

      let vtable = VTable::new::<i64>();
      let mut data = 1234_i64;
      let variant_ref = vtable.variant_ref_mut(&mut data);
      assert_eq!("1234", format!("{:?}", variant_ref));
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
   fn display_ref() {
      let vtable = VTable::new::<&str>();
      let data = "data";
      let variant = vtable.variant_ref(&data);
      assert_eq!("data", format!("{}", variant));

      let vtable = VTable::new::<i64>();
      let data = 1234_i64;
      let variant = vtable.variant_ref(&data);
      assert_eq!("1234", format!("{}", variant));
   }

   #[test]
   fn display_ref_mut() {
      let vtable = VTable::new::<&str>();
      let mut data = "data";
      let variant = vtable.variant_ref_mut(&mut data);
      assert_eq!("data", format!("{}", variant));

      let vtable = VTable::new::<i64>();
      let mut data = 1234_i64;
      let variant = vtable.variant_ref_mut(&mut data);
      assert_eq!("1234", format!("{}", variant));
   }

   #[test]
   fn deref() {
      let vtable = VTable::new::<i64>();
      let variant = vtable.variant(1234_i64);

      fn inner(variant: &VariantRef) {
         assert_eq!(Some(&1234_i64), variant.downcast_ref::<i64>());
      }

      inner(&variant);
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
