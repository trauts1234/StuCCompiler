use memory_size::MemorySize;

use crate::{args_handling::location_classification::{PreferredParamLocation, StructEightbytePreferredLocation}, asm_gen_data::GetStructUnion, assembly::operand::register::{GPRegister, MMRegister}, data_type::recursive_data_type::DataType};

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


pub fn generate_param_and_return_locations<'a, ArgIter>(arg_types: ArgIter, return_type: &DataType, get_struct_union: &dyn GetStructUnion) -> (ReturnLocation, Vec<AllocatedLocation>)
where ArgIter: IntoIterator<Item = &'a DataType>
{
    let mut arg_alloc = ArgAllocator::default();

    let return_loc = match PreferredParamLocation::param_from_type(return_type, get_struct_union) {
        //scalar - just return in the correct register
        PreferredParamLocation::InGP => ReturnLocation::InRegs(vec![EightByteLocation::GP(GPRegister::_AX)]),
        PreferredParamLocation::InMMX => ReturnLocation::InRegs(vec![EightByteLocation::XMM(MMRegister::XMM0)]),
        //multi-register - use the correct register pair
        PreferredParamLocation::Struct { l, r } => ReturnLocation::InRegs(match (l, r) {
            (StructEightbytePreferredLocation::InGP, StructEightbytePreferredLocation::InGP) => vec![EightByteLocation::GP(GPRegister::_AX), EightByteLocation::GP(GPRegister::_DX)],
            (StructEightbytePreferredLocation::InGP, StructEightbytePreferredLocation::InMMX) => vec![EightByteLocation::GP(GPRegister::_AX), EightByteLocation::XMM(MMRegister::XMM0)],
            (StructEightbytePreferredLocation::InMMX, StructEightbytePreferredLocation::InGP) => vec![EightByteLocation::XMM(MMRegister::XMM0), EightByteLocation::GP(GPRegister::_AX)],
            (StructEightbytePreferredLocation::InMMX, StructEightbytePreferredLocation::InMMX) => vec![EightByteLocation::XMM(MMRegister::XMM0), EightByteLocation::XMM(MMRegister::XMM1)],
        }),
        // here it is more tricky
        PreferredParamLocation::InMemory => {
            arg_alloc.integer_regs_used += 1;//first register is a hidden pointer
            todo!()
        },
    };

    let params_loc = 
        arg_types.into_iter()
        .map(|arg_type| arg_alloc.allocate(PreferredParamLocation::param_from_type(arg_type, get_struct_union)))
        .collect();
        

    (return_loc, params_loc)
}
    

#[derive(Default)]
struct ArgAllocator {
    pub integer_regs_used: u64,
    pub float_regs_used: u64,
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