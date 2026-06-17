// there are going to be n number of workers in here

use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crossbeam_channel::Receiver;

use crate::process::ProcessTable;

pub struct Executor {}

impl Executor {
    pub fn new(workers: usize, job_rx: Receiver<usize>, table: Arc<ProcessTable>) {
        let mut w_handles = vec![];

        for worker in 1..=workers {
            let worker_handle = Self::worker(worker, job_rx.clone(), table.clone());
            w_handles.push(worker_handle);
        }

        for h in w_handles {
            h.join().expect("error in joining the executor threads")
        }
    }

    pub fn worker(wid: usize, rx: Receiver<usize>, table: Arc<ProcessTable>) -> JoinHandle<()> {
        let jh = thread::spawn(move || {
            loop {
                let pid = rx.recv().unwrap();
                let process = table
                    .get(&pid)
                    .expect("expected to have the process with process id in the table");

                let guard = process.lock().unwrap();

                let number: u8 = rand::random();
                if number % 2 == 0 {
                    // cpu execution
                } else {
                    // io execution, means send it the blocking queue
                    // from blocking queue it will send back to the scheduler
                }

                // need to record the execution time
                // need to record the IO time and send it the blocker queue

                drop(guard);

                println!("from worker: {}, process executing with pid: {}", wid, pid);
            }
        });
        jh
    }
}
