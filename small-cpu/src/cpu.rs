pub struct Cpu {
    // registers
    rax: u64,
    rcx: u64,
    rdx: u64,
    rbx: u64,
    rsp: u64,
    rbp: u64,
    rsi: u64,
    rdi: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,

    // instruction pointer or program counter
    rip: u64,

    // stack start
    stack_ptr: u64,
    stack_limit: u64, // could be the constant

    heap_ptr: u64,

    // condition code flags
    zf: u8,
    sf: u8,
    of: u8,

    // program state
    state: ProgramState,
}

pub enum ProgramState {
    AOK, // Normal operation and all ok
    HTL, // halt instruction encountered
    ADR, // invalid address encountered
    INS, // invalid instruction encountered
}

impl Cpu {
    // load from memory
    // store in memory
    // operations
    // next-instruction
    // run the program after loading
}
