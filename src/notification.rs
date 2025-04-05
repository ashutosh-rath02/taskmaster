use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

use crate::async_executor::TaskEvent;
use crate::error::Result;

type CallbackFn = Box<dyn Fn(&TaskEvent) + Send + Sync + 'static>;

pub struct NotificationSystem {
    event_rx: mpsc::Receiver<TaskEvent>,
    callbacks: HashMap<String, CallbackFn>,
}

impl NotificationSystem {
    pub fn new(event_rx: mpsc::Receiver<TaskEvent>) -> Self {
        NotificationSystem {
            event_rx,
            callbacks: HashMap::new(),
        }
    }

    pub fn register_callback<F>(&mut self, name: &str, callback: F)
    where
        F: Fn(&TaskEvent) + Send + Sync + 'static,
    {
        self.callbacks.insert(name.to_string(), Box::new(callback));
    }

    pub fn unregister_callback(&mut self, name: &str) -> bool {
        self.callbacks.remove(name).is_some()
    }

    pub async fn start(&mut self) -> Result<()> {
        println!("Notification system started");

        while let Some(event) = self.event_rx.recv().await {
            println!("Received event: {:?}", event);

            for (name, callback) in &self.callbacks {
                println!("Executing callback: {}", name);
                callback(&event);
            }
        }

        println!("Notification system stopped");
        Ok(())
    }

    pub async fn start_with_deadlines(
        &mut self,
        tasks_with_deadlines: HashMap<u32, time::Instant>,
    ) -> Result<()> {
        println!("Notification system with deadlines started");

        let mut deadline_tasks = tasks_with_deadlines.clone();

        loop {
            tokio::select! {
                Some(event) = self.event_rx.recv() => {
                    println!("Received event: {:?}", event);

                    // Handle task completion
                    if let TaskEvent::Completed { task_id } = event {
                        deadline_tasks.remove(&task_id);
                    }

                    for (name, callback) in &self.callbacks {
                        println!("Executing callback: {}", name);
                        callback(&event);
                    }
                }
                _ = time::sleep(Duration::from_secs(1)) => {
                    // Check deadlines
                    let now = time::Instant::now();
                    let mut expired = Vec::new();

                    for (&task_id, &deadline) in &deadline_tasks {
                        if now >= deadline {
                            expired.push(task_id);
                        }
                    }

                    // Handle expired deadlines
                    for task_id in expired {
                        println!("Task {} deadline expired", task_id);

                        let event = TaskEvent::Timeout { task_id };
                        for (name, callback) in &self.callbacks {
                            println!("Executing deadline callback: {}", name);
                            callback(&event);
                        }

                        deadline_tasks.remove(&task_id);
                    }
                }
                else => break,
            }
        }

        println!("Notification system with deadlines stopped");
        Ok(())
    }
}
