use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    debug: bool,
    is_running: bool,
}

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), false));
        }
        ThreadPool {
            workers,
            sender,
            debug: false,
            is_running: true,
        }
    }

    pub fn new_with_debug(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), true));
        }
        ThreadPool {
            workers,
            sender,
            debug: true,
            is_running: true,
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if !self.is_running {
            panic!("ThreadPool shutted down.\nUsed ThreadPool::join() and then ThreadPool::execute().\nThis can not be done.");
        }
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
    pub fn join(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
        self.is_running = false;
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if self.debug {
            println!("Sending terminate message to all workers.");
        }
        if !self.is_running {
            return;
        }
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        if self.debug {
            println!("Shutting down all workers.");
        }
        for worker in &mut self.workers {
            if self.debug {
                println!("Shutting down worker {}", worker.id);
            }
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>, debug: bool) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    let start = std::time::Instant::now();
                    if debug {
                        println!("Worker {} got a job; executing.", id);
                    }
                    job();
                    let duration = start.elapsed();
                    if debug {
                        println!(
                            "Worker {} finished the job in {}ms.",
                            id,
                            duration.as_millis()
                        );
                    }
                }
                Message::Terminate => {
                    if debug {
                        println!("Worker {} was told to terminate.", id);
                    }
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let pool = ThreadPool::new(4);
        let result: u8 = 0;
        let a: u8 = 1;
        let b: u8 = 3;
        pool.execute(move || {
            let result = a + b;
            assert_eq!(result, 4);
        });

        assert_eq!(result, 0);
    }

    #[test]
    fn join() {
        let mut pool = ThreadPool::new(4);
        let result: u8 = 0;
        let a: u8 = 1;
        let b: u8 = 3;
        pool.execute(move || {
            let result = a + b;
            assert_eq!(result, 4);
        });
        pool.join();

        assert_eq!(result, 0);
    }
    #[test]
    #[should_panic]
    fn is_panic() {
        let mut pool = ThreadPool::new(4);
        let result: u8 = 0;
        let a: u8 = 1;
        let b: u8 = 3;
        pool.execute(move || {
            let result = a + b;
            assert_eq!(result, 4);
        });
        pool.join();
        pool.execute(move || {
            let result = a + b;
            assert_eq!(result, 4);
        });

        assert_eq!(result, 0);
    }
}
