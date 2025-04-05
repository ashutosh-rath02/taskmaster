use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use crate::error::{Result, TaskMasterError};
use crate::task::Task;

// Message types for the worker pool
enum Message {
    NewTask(TaskJob),
    Terminate,
}

// A job to be executed by the worker pool
pub struct TaskJob {
    pub id: u32,
    pub task: Arc<Task>,
    pub handler: Box<dyn FnOnce(Arc<Task>) -> Result<()> + Send + 'static>,
}

// Result of a completed job
pub struct JobResult {
    pub task_id: u32,
    pub success: bool,
    pub error_message: Option<String>,
}

// The worker pool
pub struct WorkerPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
    results_sender: mpsc::Sender<JobResult>,
    results_receiver: mpsc::Receiver<JobResult>,
}

impl WorkerPool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let (results_sender, results_receiver) = mpsc::channel();

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                results_sender.clone(),
            ));
        }

        WorkerPool {
            workers,
            sender,
            receiver,
            results_sender,
            results_receiver,
        }
    }

    pub fn execute(&self, job: TaskJob) -> Result<()> {
        self.sender.send(Message::NewTask(job)).map_err(|_| {
            TaskMasterError::InvalidOperation("Worker pool is disconnected".to_string())
        })?;
        Ok(())
    }

    pub fn get_result(&self) -> Result<JobResult> {
        self.results_receiver.recv().map_err(|_| {
            TaskMasterError::InvalidOperation("Result channel is disconnected".to_string())
        })
    }

    pub fn try_get_result(&self) -> Option<JobResult> {
        self.results_receiver.try_recv().ok()
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// A worker in the pool
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
        results_sender: mpsc::Sender<JobResult>,
    ) -> Self {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewTask(job) => {
                    println!("Worker {} got a job; executing.", id);

                    let task_id = job.id;
                    let result = (job.handler)(job.task);

                    let job_result = match result {
                        Ok(_) => JobResult {
                            task_id,
                            success: true,
                            error_message: None,
                        },
                        Err(e) => JobResult {
                            task_id,
                            success: false,
                            error_message: Some(e.to_string()),
                        },
                    };

                    results_sender.send(job_result).unwrap();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
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
