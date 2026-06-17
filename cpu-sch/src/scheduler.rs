use std::{
    sync::{Arc, Mutex},
    thread,
};

use crossbeam_channel::{Receiver, Sender};

use crate::{dll::LinkedList, process::ProcessTable};

pub struct Queue {
    pub level: usize, // lower means the high priority
    pub ll: Arc<Mutex<LinkedList<usize>>>,
    pub quanta: usize, // in millis, and used for how much time the task will sit at the same level in the queue
}

impl Queue {
    fn new(level: usize) -> Self {
        Self {
            level,
            ll: Arc::new(Mutex::new(LinkedList::new())),
            quanta: level * 10,
        }
    }
}

pub struct Scheduler {
    pub queues: Vec<Queue>,
    pub table: Arc<ProcessTable>, // will have some channels
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

        let first_queue = queues.get(0).unwrap();
        let mut guard = first_queue.ll.lock().expect("error in locking the queue");

        // seed jobs all are pushed at the first level
        for id in table.0.keys() {
            guard.push_front(*id);
            job_sender.send(*id).unwrap();
        }
        drop(guard);

        // thread for the job sender and which can current thread
        let queues = Arc::new(queues);
        Self::handle_io_jobs(table.clone(), queues.clone(), io_job_rx);

        // jobs sender, which read jobs from mlfq and then send them to sender

        loop {
            // todo: for now, let's execute the job level wise one by one, later we have to change this logic
            // if there is any job in the low priority give that the priority first to execute

            for queue in queues.iter() {
                let mut ll = queue.ll.lock().expect("error in the job execution"); // issue is that, consuming the whole queue at once
                while let Some(pid) = ll.pop_back() {
                    job_sender
                        .send(pid)
                        .expect("error in sending the job back to the queue")
                }
                drop(ll);
            }
        }
    }

    // io queue receiver thread
    // must be separate thread and after sometime it should return the job back at the same level
    fn handle_io_jobs(
        table: Arc<ProcessTable>,
        queues: Arc<Vec<Queue>>,
        io_job_rx: Receiver<usize>,
    ) {
        thread::spawn(move || {
            loop {
                let pid = io_job_rx.recv().unwrap();
                Self::push_job_in_queue(&table, &queues, pid);
            }
        });
    }

    // this method inserts the job in the queue
    // in the time quanta reached then it decreases the level of the queue
    // and if the job is about to go out of array bounds, it push back the job to top level queue
    pub fn push_job_in_queue(table: &Arc<ProcessTable>, queues: &Arc<Vec<Queue>>, pid: usize) {
        let process = table
            .get(&pid)
            .expect("expected the process to push back it in the job queue");
        let mut process_guard = process.lock().expect("error in locking");

        let queue = queues
            .get(process_guard.level)
            .expect("error in getting the level");

        if process_guard.total_time_at_level >= queue.quanta {
            // decrease the priority of it
            if process_guard.level + 1 >= queues.len() {
                // boost the prority of the process
                process_guard.level = 0;
                println!("????????????? priority boosted");
            } else {
                println!("############ priority decreased");
                process_guard.level += 1;
            }
            process_guard.total_time_at_level = 0;
        } else {
            // insert at the same level if the level time quanta is not reached
        }

        assert!(
            process_guard.level < queues.len(),
            "level must be lesser then queue len"
        );
        let queue = queues
            .get(process_guard.level)
            .expect("error in getting the level");

        queue.ll.lock().unwrap().push_front(pid);
        drop(process_guard);
    }
}
