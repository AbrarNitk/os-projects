const STACK_SIZE: usize = 1024;

pub struct Stack {
    stack: [u8; STACK_SIZE],
    stack_ptr: usize,
}
