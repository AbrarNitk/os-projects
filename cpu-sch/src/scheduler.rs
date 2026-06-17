use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender};

use crate::{dll::LinkedList, process::ProcessTable};

pub struct Queue {
    pub level: usize, // lower means the high priority
    pub ll: Mutex<LinkedList<usize>>,
    pub quanta: usize, // in millis, and used for how much time the task will sit at the same level in the queue
}

impl Queue {
    fn new(level: usize) -> Self {
        Self {
            level,
            ll: Mutex::new(LinkedList::new()),
            quanta: level * 10,
        }
    }
}

pub struct Scheduler {
    pub queues: Vec<Queue>,
    // will have some channels
}

impl Scheduler {
    pub fn new(
        total_queue: usize,
        table: Arc<ProcessTable>,
        job_sender: Sender<usize>,
        io_job_rx: Receiver<usize>,
    ) {
        let mut queues = Vec::with_capacity(total_queue);
        for level in 1..=total_queue {
            let q = Queue::new(level);
            queues.push(q);
        }

        // sender thread
        for id in table.0.keys() {
            job_sender.send(*id).unwrap();
        }

        // io queue receiver thread
        // must be separate thread and after sometime it should return the job back at the same level
        loop {
            let pid = io_job_rx.recv().unwrap();
            job_sender.send(pid).unwrap();
        }
    }
}
