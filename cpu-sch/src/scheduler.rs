use std::sync::Arc;

use crossbeam_channel::Sender;

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
    pub fn new(sender: Sender<usize>, table: Arc<ProcessTable>) {
        for id in table.0.keys() {
            sender.send(*id).unwrap();
        }

        loop {
            sender.send(1).unwrap();
        }
    }
}
