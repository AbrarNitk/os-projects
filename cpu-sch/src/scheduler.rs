use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};

use crate::{dll::LinkedList, process::ProcessTable};

pub struct Queue {
    pub level: usize, // lower means the high priority
    pub ll: LinkedList<usize>,
    pub quanta: usize, // in millis, and used for how much time the task will sit at the same level in the queue
}

pub struct Scheduler {
    pub queues: Vec<Queue>,
    // will have some channels
}

impl Scheduler {
    pub fn new(table: Arc<ProcessTable>, job_sender: Sender<usize>, io_job_rx: Receiver<usize>) {
        for id in table.0.keys() {
            job_sender.send(*id).unwrap();
        }

        loop {
            let pid = io_job_rx.recv().unwrap();
            job_sender.send(pid).unwrap();
        }
    }
}
