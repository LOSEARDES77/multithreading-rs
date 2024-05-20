use std::marker::PhantomData;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool<T> {
    workers: Vec<Worker<T>>,
    sender: mpsc::Sender<Message<T>>,
    debug: bool,
}

pub trait FnPool<T> {
    fn call(self) -> T;
}

impl<F, T> FnPool<T> for F
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
{
    fn call(self) -> T {
        self()
    }
}

enum Message<T> {
    NewJob(Job<T>, mpsc::Sender<T>),
    Terminate,
}

impl<T: Send + 'static> ThreadPool<T> {
    pub fn new(size: usize) -> ThreadPool<T> {
        let mut size = size;
        if size == 0 {
            size = num_cpus::get();
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers: Vec<Worker<T>> = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), false));
        }
        ThreadPool {
            workers,
            sender,
            debug: false,
        }
    }

    pub fn new_with_debug(size: usize) -> ThreadPool<T> {
        let mut size = size;
        if size == 0 {
            size = num_cpus::get();
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers: Vec<Worker<T>> = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), true));
        }
        ThreadPool {
            workers,
            sender,
            debug: true,
        }
    }

    pub fn execute<F>(&self, f: F) -> mpsc::Receiver<T>
        where
            F: FnOnce() -> T + Send + 'static,
            T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let job = Box::new(f);
        self.sender
            .send(Message::NewJob(job, tx))
            .expect("Failed to send job to thread pool");
        rx
    }
}

impl<T> Drop for ThreadPool<T> {
    fn drop(&mut self) {
        if self.debug {
            println!("Sending terminate message to all workers.");
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

type Job<T> = Box<dyn FnOnce() -> T + Send + 'static>;

#[warn(dead_code)]
struct Worker<T> {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
    return_type: PhantomData<T>,
}

impl<T: Send + 'static> Worker<T> {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message<T>>>>, debug: bool) -> Worker<T> {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job, tx) => {
                    let result = job();
                    tx.send(result)
                        .expect("Failed to send result back to main thread");
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
            return_type: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer() {
        let pool = ThreadPool::new(0);
        let a: u8 = 1;
        let b: u8 = 3;
        let receiver = pool.execute(move ||{
            a + b
        });

        let result = receiver.recv().unwrap();
        assert_eq!(result, 4);
    }

    #[test]
    fn test_string() {
        let pool = ThreadPool::new(0);
        let a = String::from("Hello");
        let b = String::from("World");
        let receiver = pool.execute(move ||{
            a + " " + &b
        });

        let result = receiver.recv().unwrap();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_struct() {
        struct Test {
            a: u8,
            b: u8,
        }

        let pool = ThreadPool::new(0);
        let a = Test { a: 123, b: 86 };
        let receiver = pool.execute(move ||{
            a.a + a.b
        });

        let result = receiver.recv().unwrap();
        assert_eq!(result, 209);
    }

    #[test]
    fn test_closure() {
        let pool = ThreadPool::new(0);
        let a: u8 = 1;
        let b: u8 = 3;
        let receiver = pool.execute(||{
            a + b
        });

        let result = receiver.recv().unwrap();
        assert_eq!(result, 4);
    }
}
