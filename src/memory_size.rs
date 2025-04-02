use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, PartialEq, Clone, Copy)]//remove Copy?
pub struct MemoryLayout {
    size_bits: usize
}

impl MemoryLayout {
    /**
     * Creates a MemorySize with a size of 0
     */
    pub const fn new() -> MemoryLayout {
        MemoryLayout{
            size_bits: 0
        }
    }
    /**
     * Construct a MemorySize from a number of bytes
     */
    pub const fn from_bytes(bytes: usize) -> MemoryLayout{
        MemoryLayout{
            size_bits: bytes*8
        }
    }
    /**
     * Construct a MemorySize from number of bits
     */
    pub const fn from_bits(bits: usize) -> MemoryLayout{
        MemoryLayout{
            size_bits:bits
        }
    }

    pub fn biggest(lhs: &MemoryLayout, rhs: &MemoryLayout) -> MemoryLayout {
        MemoryLayout::from_bits(
            lhs.size_bits().max(rhs.size_bits())
        )
    }

    /**
     * Calculate the size suggested by this MemorySize in bytes
     */
    pub fn size_bytes(&self) -> usize{
        let rounded_down_ans = self.size_bits/8;
        let remaining_bits = self.size_bits%8;

        //add one if bits are left over, so that there are enough bytes to store all the bits
        if remaining_bits > 0{
            rounded_down_ans + 1
        } else{
            rounded_down_ans
        }
    }

    /**
     * Calculate the size suggested by this MemorySize in bits
     */
    pub fn size_bits(&self) -> usize{
        self.size_bits
    }

}

impl Add for MemoryLayout {
    type Output = MemoryLayout;

    fn add(self, rhs: MemoryLayout) -> MemoryLayout {
        MemoryLayout::from_bits(self.size_bits+rhs.size_bits)
    }
}

impl AddAssign for MemoryLayout {
    fn add_assign(&mut self, rhs: MemoryLayout) {
        self.size_bits += rhs.size_bits;
    }
}

impl Sub for MemoryLayout {
    type Output = MemoryLayout;

    fn sub(self, rhs: MemoryLayout) -> MemoryLayout {
        MemoryLayout::from_bits(self.size_bits-rhs.size_bits)
    }
}

impl SubAssign for MemoryLayout {
    fn sub_assign(&mut self, rhs: MemoryLayout) {
        self.size_bits -= rhs.size_bits;
    }
}

impl MemoryLayout {
    /**
     * Sets self to the biggest of self and rhs
     */
    pub fn set_to_biggest(&mut self, rhs: MemoryLayout) {
        if rhs.size_bits() > self.size_bits() {
            self.size_bits = rhs.size_bits();
        }
    }
}