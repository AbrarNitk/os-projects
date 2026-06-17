// there are going to be n number of workers in here

use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::{Receiver, Sender};

use crate::process::ProcessTable;

pub struct Executor {}

impl Executor {
    pub fn new(
        workers: usize,
        table: Arc<ProcessTable>,
        job_rx: Receiver<usize>,
        io_job_tx: Sender<usize>,
    ) {
        let mut w_handles = vec![];

        for worker in 1..=workers {
            let worker_handle =
                Self::worker(worker, table.clone(), job_rx.clone(), io_job_tx.clone());
            w_handles.push(worker_handle);
        }

        for h in w_handles {
            h.join().expect("error in joining the executor threads")
        }
    }

    pub fn worker(
        wid: usize,
        table: Arc<ProcessTable>,
        rx: Receiver<usize>,
        io_job_tx: Sender<usize>,
    ) -> JoinHandle<()> {
        let jh = thread::spawn(move || {
            loop {
                let pid = rx.recv().unwrap();
                println!("from worker: {}, process executing with pid: {}", wid, pid);

                let process = table
                    .get(&pid)
                    .expect("expected to have the process with process id in the table");

                // this is dangerous, we are saying when the process is
                // getting executed no one is allowed to touch it
                // this is definitely not the way to implement a part of the Operating System

                let mut guard = process.lock().unwrap();
                let can_cpu = if guard.cpu_time > 0 { true } else { false };
                let can_io = if guard.total_io_ops > 0 { true } else { false };

                println!(
                    "from worker-before execution: {}, pid: {}, cpu-time: {}, ios: {}",
                    wid, pid, guard.cpu_time, guard.total_io_ops
                );

                if can_cpu && can_io {
                    let number: u8 = rand::random();
                    if number % 2 == 0 {
                        // cpu gave 10ms to each cpu process
                        // cpu execution
                        if guard.cpu_time > 10 {
                            thread::sleep(Duration::from_millis(100));
                            guard.cpu_time -= 10;
                            guard.total_time_at_level += 10;
                            io_job_tx.send(pid).expect("io job send error");
                        } else {
                            thread::sleep(Duration::from_millis((guard.cpu_time * 10) as u64));
                            guard.cpu_time = 0;
                            println!("process cpu executed successfully: {}", pid);
                            // means the execution is done, no need tro schedule it further
                        }
                    } else {
                        if guard.total_io_ops > 1 {
                            guard.total_io_ops -= 1;
                            // send back to the scheduler
                            io_job_tx.send(pid).expect("io job send error");
                        } else {
                            println!("process executed successfully: {}", pid);
                        }
                        // io execution, means send it the blocking queue
                        // from blocking queue it will send back to the scheduler
                    }
                } else {
                    if can_cpu {
                        if guard.cpu_time > 10 {
                            thread::sleep(Duration::from_millis(100));
                            guard.cpu_time -= 10;
                            guard.total_time_at_level += 10;
                            io_job_tx.send(pid).expect("io job send error");
                        } else {
                            thread::sleep(Duration::from_millis((guard.cpu_time * 10) as u64));
                            guard.cpu_time = 0;
                            println!("process executed successfully: {}", pid);
                            // means the execution is done, no need tro schedule it further
                        }
                    }

                    if can_io {
                        if guard.total_io_ops > 1 {
                            guard.total_io_ops -= 1;
                            // send back to the scheduler
                            io_job_tx.send(pid).expect("io job send error");
                        } else {
                            println!("process executed successfully: {}", pid);
                        }
                    }
                }

                println!(
                    "from worker after execution: {}, pid: {}, cpu-time: {}, ios: {}",
                    wid, pid, guard.cpu_time, guard.total_io_ops
                );

                drop(guard);
            }
        });
        jh
    }
}
