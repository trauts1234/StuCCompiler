use memory_size::MemorySize;

use crate::{args_handling::location_classification::{PreferredParamLocation, StructEightbytePreferredLocation}, assembly::operand::register::{GPRegister, MMRegister}, data_type::recursive_data_type::DataType, declaration::Declaration};

const MAX_GP_REGS: u64 = 6;
const MAX_XMM_REGS: u64 = 8;

#[derive(Debug, Clone, PartialEq)]
pub enum EightByteLocation {
    GP(GPRegister),
    XMM(MMRegister)
}

#[derive(Debug, Clone, PartialEq)]
pub enum AllocatedLocation {
    /// The vec stores the register locations of each eightbyte of the data
    Regs(Vec<EightByteLocation>),
    /// add to RSP for the address of it
    Memory,
}

#[derive(Clone)]
pub enum ReturnLocation {
    InRegs(Vec<EightByteLocation>),
    // pointer to the return data is stored at [rbp-pointer_bp_offset]
    InMemory {pointer_bp_offset: MemorySize}
}

/// Stores where the args should be as per the ABI
#[derive(Clone)]
pub struct AbiArgs {
    return_loc: ReturnLocation,
    params_loc: Vec<AllocatedLocation> 
}
impl AbiArgs {
    pub fn generate(args: &[Declaration], return_type: &DataType) -> Self {
        let return_loc = match return_type {

        }
    }
    pub fn return_location(&self) -> &ReturnLocation {
        &self.return_loc
    }
    pub fn param_location(&self, param_num: usize) -> &AllocatedLocation {
        &self.params_loc[param_num]
    }
}

pub struct ArgAllocator {
    integer_regs_used: u64,
    float_regs_used: u64,
}
impl ArgAllocator {
    /// Since some return values are passed via hidden pointer...
    pub fn new(func_return_preferred_location: PreferredParamLocation) -> (Self, AllocatedLocation) {
        let mut result = Self {
            integer_regs_used: 0,
            float_regs_used: 0,
        };


        let ret_location = match func_return_preferred_location {
            //single GP returns in RAX
            PreferredParamLocation::InGP => AllocatedLocation::Regs(vec![EightByteLocation::GP(GPRegister::_AX)]),
            //caller needs to reserve stack, and put a hidden pointer in the first param
            PreferredParamLocation::InMemory => {
                result.allocate(PreferredParamLocation::InGP)
            },
            //single XMM returns in XMM0
            PreferredParamLocation::InMMX => AllocatedLocation::Regs(vec![EightByteLocation::XMM(MMRegister::XMM0)]),
            //struct in 2 parts returns in 2 appropriate return registers
            PreferredParamLocation::Struct { l, r } => {
                let return_gp = [GPRegister::_AX, GPRegister::_DX];
                let return_xmm = [MMRegister::XMM0, MMRegister::XMM1];
                let mut allocated = Vec::new();
                //allocate each eightbyte based on *return location*
                for eightbyte in [l, r] {
                    match eightbyte {
                        StructEightbytePreferredLocation::InGP => {
                            allocated.push(EightByteLocation::GP(return_gp[result.integer_regs_used as usize]));
                            result.integer_regs_used += 1;
                        },
                        StructEightbytePreferredLocation::InMMX => {
                            allocated.push(EightByteLocation::XMM(return_xmm[result.float_regs_used as usize]));
                            result.float_regs_used += 1;
                        },
                    }
                }
                AllocatedLocation::Regs(allocated)
            }
        };

        (result, ret_location)
    } 
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