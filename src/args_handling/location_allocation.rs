use crate::{args_handling::location_classification::{PreferredParamLocation, StructEightbytePreferredLocation}, assembly::operand::register::{GPRegister, MMRegister}};

const MAX_GP_REGS: u64 = 6;
const MAX_XMM_REGS: u64 = 8;

#[derive(Debug)]
pub enum EightByteLocation {
    GP(GPRegister),
    XMM(MMRegister)
}

#[derive(Debug)]
pub enum AllocatedLocation {
    /// The vec stores the register locations of each eightbyte of the data
    Regs(Vec<EightByteLocation>),
    /// This variant deliberately does not specify where in memory, to allow the calling code to handle this
    Memory,
}

#[derive(Default)]
pub struct ArgAllocator {
    integer_regs_used: u64,
    float_regs_used: u64,
}
impl ArgAllocator {
    /**
     * Allocates registers for params
     */
    pub fn allocate(&mut self, preferred_location: PreferredParamLocation) -> AllocatedLocation {
        match preferred_location {
            PreferredParamLocation::InGP if self.integer_regs_used < MAX_GP_REGS => {
                let result = AllocatedLocation::Regs(vec![EightByteLocation::GP(gp_arg(self.integer_regs_used))]);
                self.integer_regs_used += 1;
                result
            },
            PreferredParamLocation::InMMX if self.float_regs_used < MAX_XMM_REGS => {
                let result = AllocatedLocation::Regs(vec![EightByteLocation::XMM(xmm_arg(self.float_regs_used))]);
                self.float_regs_used += 1;
                result
            },
            PreferredParamLocation::Struct { l, r } if self.can_alloc_these(&l, &r) => {
                let mut result = Vec::new();
                //allocate each eightbyte
                for eightbyte in [l, r] {
                    match eightbyte {
                        StructEightbytePreferredLocation::InGP => {
                            result.push(EightByteLocation::GP(gp_arg(self.integer_regs_used)));
                            self.integer_regs_used += 1;
                        },
                        StructEightbytePreferredLocation::InMMX => {
                            result.push(EightByteLocation::XMM(xmm_arg(self.float_regs_used)));
                            self.float_regs_used += 1;
                        },
                    }
                }
                AllocatedLocation::Regs(result)

            }
            _ => {
                //must be memory then
                AllocatedLocation::Memory
            },
        }
    }

    /// Calculates whether I have the registers left to allocate these
    fn can_alloc_these(&self, l: &StructEightbytePreferredLocation, r: &StructEightbytePreferredLocation) -> bool {
        let gp_regs_required: u64 =
            [l, r].iter()
            .filter(|x| ***x == StructEightbytePreferredLocation::InGP)
            .count()
            .try_into()
            .unwrap();
        let xmm_regs_required: u64 =
            [l, r].iter()
            .filter(|x| ***x == StructEightbytePreferredLocation::InMMX)
            .count()
            .try_into()
            .unwrap();

        (self.integer_regs_used + gp_regs_required) <= MAX_GP_REGS &&
        (self.float_regs_used + xmm_regs_required) <= MAX_XMM_REGS

    }
}

fn gp_arg(idx: u64) -> GPRegister {
    match idx {
        0 => GPRegister::_DI,//starts at 1 because I have already incremented the counter
        1 => GPRegister::_SI,
        2 => GPRegister::_DX,
        3 => GPRegister::_CX,
        4 => GPRegister::R8,
        5 => GPRegister::R9,
        _ => panic!("this param should be on the stack.")
    }
}
fn xmm_arg(idx: u64) -> MMRegister {
    match idx {
        0 => MMRegister::XMM0,
        1 => MMRegister::XMM1,
        2 => MMRegister::XMM2,
        3 => MMRegister::XMM3,
        4 => MMRegister::XMM4,
        5 => MMRegister::XMM5,
        6 => MMRegister::XMM6,
        7 => MMRegister::XMM7,

        8.. => unreachable!()
    }
}