
/**
 * stores a possible memory type, like on the stack or in the AX register
 */
pub enum MemoryType {
    _AX,//rax or eax
    PUSHTOSTACK,//increment stack and put on top
}