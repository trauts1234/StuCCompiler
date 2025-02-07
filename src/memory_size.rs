use std::ops::{Add, Sub, Neg, AddAssign, SubAssign};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MemoryLayout {
    size_bits: isize
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
    pub const fn from_bytes(bytes: isize) -> MemoryLayout{
        MemoryLayout{
            size_bits: bytes*8
        }
    }
    /**
     * Construct a MemorySize from number of bits
     */
    pub const fn from_bits(bits: isize) -> MemoryLayout{
        MemoryLayout{
            size_bits:bits
        }
    }

    /**
     * Calculate the size suggested by this MemorySize in bytes
     */
    pub fn size_bytes(&self) -> isize{
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
     * returns true if the size suggested by this MemorySize in bits is greater than 0
     */
    pub fn is_positive(&self) -> bool{
        return self.size_bits > 0;
    }

    /**
     * Calculate the size suggested by this MemorySize in bits
     */
    pub fn size_bits(&self) -> isize{
        self.size_bits
    }

    pub fn safe_cast_to_unsigned(x: isize) -> usize {
        if x.is_negative() {
            panic!("tried to cast negative number to unsigned")
        } else{
            x as usize
        }
    }
    pub fn safe_cast_to_signed(x: usize) -> isize{
        if x > isize::MAX.try_into().unwrap() {
            panic!("tried to cast too large number to signed")
        } else {
            x as isize
        }
    }

}

impl Add for MemoryLayout {
    type Output = MemoryLayout;

    fn add(self, rhs: MemoryLayout) -> MemoryLayout {
        if self.size_bits.is_negative() || rhs.size_bits.is_negative(){
            eprintln!("tried to add memory sizes, with one arg being a negative size")
        }
        MemoryLayout::from_bits(self.size_bits+rhs.size_bits)
    }
}

impl AddAssign for MemoryLayout {
    fn add_assign(&mut self, rhs: MemoryLayout) {
        if rhs.size_bits.is_negative(){
            eprintln!("adding negative size to memory size")
        }
        self.size_bits += rhs.size_bits;
    }
}

impl Sub for MemoryLayout {
    type Output = MemoryLayout;

    fn sub(self, rhs: MemoryLayout) -> MemoryLayout {
        if rhs.size_bits.is_negative(){
            eprintln!("subtracting negative size from memory size")
        }
        MemoryLayout::from_bits(self.size_bits-rhs.size_bits)
    }
}

impl SubAssign for MemoryLayout {
    fn sub_assign(&mut self, rhs: MemoryLayout) {
        if rhs.size_bits.is_negative(){
            eprintln!("subtracting negative size from memory size")
        }
        self.size_bits -= rhs.size_bits;
    }
}

impl Neg for MemoryLayout {
    type Output = MemoryLayout;

    fn neg(self) -> MemoryLayout {
        MemoryLayout::from_bits(-self.size_bits)
    }
}