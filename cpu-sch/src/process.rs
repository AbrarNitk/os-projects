use std::{collections::HashMap, ops::Deref, sync::Mutex};

#[derive(Debug)]
pub struct Process {
    pub id: usize,
    pub cpu_time: usize,
    pub total_io_ops: usize,
    // kernel related options
    pub total_time_at_level: usize,
    pub level: usize,
    // later we can have registers, memory and other things in here
}
pub struct ProcessTable(pub HashMap<usize, Mutex<Process>>);

impl ProcessTable {
    pub fn seed(n: usize) -> Self {
        let mut table = HashMap::new();

        let mut cpu_time_iter = std::iter::repeat_with(|| rand::random_range(10..=100));
        let mut io_number_iter = std::iter::repeat_with(|| rand::random_range(0..=10));

        let mut pid = 1;

        for _ in 0..n {
            let p = Process {
                id: pid,
                cpu_time: cpu_time_iter.next().unwrap(),
                total_io_ops: io_number_iter.next().unwrap(),

                // for kernel tracking
                total_time_at_level: 0,
                level: 0,
            };

            table.insert(pid, Mutex::new(p));
            pid += 1;
        }
        Self(table)
    }
}

impl Deref for ProcessTable {
    type Target = HashMap<usize, Mutex<Process>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// we may haver the kernel in here to maintain all the processes
// but for now that okay, for now it just create an extra layer of abstraction for us
