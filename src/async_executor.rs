use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tokio::time;

use crate::error::{Result, TaskMasterError};
use crate::task::{Task, TaskStatus};

#[derive(Debug, Clone)]
pub enum TaskEvent {
    Started { task_id: u32 },
    Completed { task_id: u32 },
    Failed { task_id: u32, error_message: String },
    Timeout { task_id: u32 },
    Terminated { task_id: u32 },
}

pub struct AsyncTaskExecutor {
    running_tasks: Arc<Mutex<HashMap<u32, Instant>>>,
    timeout: Duration,
    event_tx: mpsc::Sender<TaskEvent>,
    event_rx: Arc<Mutex<mpsc::Receiver<TaskEvent>>>,
}

impl AsyncTaskExecutor {
    pub fn new(timeout_seconds: u64, channel_capacity: usize) -> Self {
        let (event_tx, event_rx) = mpsc::channel(channel_capacity);

        AsyncTaskExecutor {
            running_tasks: Arc::new(Mutex::new(HashMap::new())),
            timeout: Duration::from_secs(timeout_seconds),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
        }
    }

    pub async fn execute_task(&self, task: Task) -> Result<()> {
        let task_id = task.id;

        {
            let mut running = self.running_tasks.lock().unwrap();
            running.insert(task_id, Instant::now());
        }

        let running_tasks = Arc::clone(&self.running_tasks);
        let event_tx = self.event_tx.clone();

        // Send started event
        event_tx
            .send(TaskEvent::Started { task_id })
            .await
            .map_err(|_| {
                TaskMasterError::ChannelError("Failed to send task started event".to_string())
            })?;

        // Spawn a new task
        tokio::spawn(async move {
            // Simulate task execution
            println!("Async executing task: {}", task.title);
            time::sleep(Duration::from_secs(2)).await;

            // Mark task as completed
            {
                let mut running = running_tasks.lock().unwrap();
                running.remove(&task_id);
            }

            // Send completed event
            let _ = event_tx.send(TaskEvent::Completed { task_id }).await;
        });
        Ok(())
    }

    pub async fn cancel_task(&self, task_id: u32) -> Result<()> {
        let mut running = self.running_tasks.lock().unwrap();
        if running.remove(&task_id).is_some() {
            Ok(())
        } else {
            Err(TaskMasterError::TaskNotFound(task_id))
        }
    }

    pub fn check_timeouts(&self) -> Vec<u32> {
        let mut running = self.running_tasks.lock().unwrap();
        let now = Instant::now();

        let timed_out: Vec<u32> = running
            .iter()
            .filter(|(_, start_time)| now.duration_since(**start_time) > self.timeout)
            .map(|(id, _)| *id)
            .collect();

        for id in &timed_out {
            running.remove(id);
        }

        timed_out
    }

    pub async fn next_event(&self) -> Option<TaskEvent> {
        let mut rx = self.event_rx.lock().unwrap();
        rx.recv().await
    }

    pub fn is_task_running(&self, task_id: u32) -> bool {
        let running = self.running_tasks.lock().unwrap();
        running.contains_key(&task_id)
    }
}
