use std::{
    io::{Seek, SeekFrom, Write},
    sync::Arc,
    thread::JoinHandle,
};

use crate::zip::pager::Pager;

pub struct ThreadPool {
    // need one file output file pager
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(workers: usize, pager: Pager, writer: &str) -> Self {
        let pager = Arc::new(pager);
        let ws = (0..workers)
            .into_iter()
            .map(|id| Worker::new(id + 1, &pager, writer))
            .collect();

        Self { workers: ws }
    }

    pub fn execute(self) {
        let mut handles = vec![];
        for worker in self.workers {
            let handle = worker.spawn();
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("error in joining the worker thread");
        }
    }
}

pub struct Worker {
    id: usize,
    pager: Arc<Pager>,
    writer: std::fs::File,
}

impl Worker {
    pub fn new(id: usize, pager: &Arc<Pager>, writer: &str) -> Self {
        Self {
            id,
            pager: Arc::clone(pager),
            writer: std::fs::File::options()
                .read(true)
                .write(true)
                .open(writer)
                .expect("error in openning a file"),
        }
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let buf = [('a' as u8) + self.id as u8; 4096];

            loop {
                // keep reading files from shared channel
                let page = self.pager.next_page();
                std::println!("thread=>{} page=>{}:{}", self.id, page.0, page.1);
                self.writer
                    .seek(SeekFrom::Start(page.0 as u64))
                    .expect("error in seeking");

                self.writer
                    .write(&buf)
                    .expect("error in writing in the file");
                // std::thread::sleep(std::time::Duration::from_micros(500));
            }
        })
    }
}

#[cfg(test)]
mod test {
    use crate::zip::pager::Pager;

    #[test]
    fn test() {
        let worker_pool = super::ThreadPool::new(4, Pager::new(), "temp.txt");
        worker_pool.execute();
    }
}
