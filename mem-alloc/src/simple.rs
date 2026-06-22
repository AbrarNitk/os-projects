// simple buddy allocator

use std::cell::RefCell;
use std::rc::Rc;

pub struct BuddyNode {
    pub offset: usize,
    pub next: Node,
}

type Node = Option<Rc<RefCell<BuddyNode>>>;

pub struct Buddy {
    buddies: [Node; 32],
    available: usize,
    size: usize,
    last: usize,
}

const fn num_bits<T>() -> u32 {
    (size_of::<T>() << 3) as u32
}

fn get_idx(x: usize) -> usize {
    assert!(x > 0);
    (num_bits::<usize>() - (x - 1).leading_zeros()) as usize
}

impl Buddy {
    pub fn init(size: usize) {
        let mut idx = get_idx(size);

        println!("size idx - {}", 1 << idx)
    }
}

#[cfg(test)]
pub mod tests {
    #[test]
    fn test() {
        super::Buddy::init(1025);
    }
}
