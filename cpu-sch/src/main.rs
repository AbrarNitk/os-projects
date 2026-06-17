use crossbeam_channel::unbounded;

use crate::scheduler::Scheduler;

pub mod dll;
pub mod executor;
pub mod io;
pub mod process;
pub mod scheduler;

fn main() {
    // create a N number of processes with each having random time to execute
    //
    let processes = process::ProcessTable::seed(1000);

    let table = std::sync::Arc::new(processes);

    let (sender, receiver) = unbounded();

    // need to spawn the scheduler by passing the table
    let s_table = table.clone();
    let scheduler_handler = std::thread::spawn(move || {
        Scheduler::new(sender, s_table);
    });

    let e_table = table.clone();
    let executor_handler = std::thread::spawn(move || {
        executor::Executor::new(10, receiver, e_table);
    });

    scheduler_handler.join().unwrap();
    executor_handler.join().unwrap();

    // todo: here keep creating the new jobs

    // pass the process table to the scheduler
    // and where it reads all the processes
}
