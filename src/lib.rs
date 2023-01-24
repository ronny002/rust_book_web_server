use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>
}

enum Message {
    NewJob(Job),
    Terminate
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (tx, rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        let mut workers = vec![];
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&rx)));
        }
        ThreadPool {
            workers,
            sender: tx,
        }
    }

    pub fn execute<T>(&self, f: T)
    where
        T: FnOnce() + Send + 'static,
    {
        self.sender.send(Message::NewJob(Box::new(f))).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Shutting down all workers");
        for _ in &self.workers{
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = rx.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Message::Terminate => {
                    println!("Worker {id} disconnecting, shutdown");
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
