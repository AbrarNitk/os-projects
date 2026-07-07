use std::sync::atomic::AtomicUsize;

// thread safe pager
pub struct Pager {
    pub total: AtomicUsize,
}

// takes a file and provides a pager onto it
// and later we map file inode to page and file-name to the inode
impl Pager {
    const PAGE_SIZE: usize = 4 * 1024;

    pub fn new() -> Self {
        Self {
            total: AtomicUsize::new(0),
        }
    }

    pub fn next_page(&self) -> (usize, usize) {
        (
            self.total
                .fetch_add(Self::PAGE_SIZE - 1, std::sync::atomic::Ordering::Relaxed),
            Self::PAGE_SIZE,
        )
    }
}

unsafe impl Sync for Pager {}
unsafe impl Send for Pager {}
