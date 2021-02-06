/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # BitSet
//!
//! Utility to iterate over the bit positions of the bits set in an u32

use core::iter::Iterator;

/// A Wrapper around an u32 value that allows iterating over the bits set in the inner value
pub struct BitSet32(pub(crate) u32);

impl BitSet32 {
  pub fn iter(&self) -> BitSet32Iter {
    BitSet32Iter(self.0)
  }
}

/// An iterator that yields the positions of the bits set for the inner value
pub struct BitSet32Iter(u32);

impl Iterator for BitSet32Iter {
  type Item = u32;

  fn next(&mut self) -> Option<Self::Item> {
    if self.0 == 0 {
      None
    } else {
      let pos = self.0.trailing_zeros();
      self.0 &= !(1 << pos);
      Some(pos)
    }
  }
}
