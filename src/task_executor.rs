use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{Result, TaskMasterError};
use crate::task::{Task};
use crate::worker_pool::{JobResult, TaskJob, WorkerPool};

pub struct TaskExecutor {
    worker_pool: WorkerPool,
    running_tasks: Arc<Mutex<HashMap<u32, Instant>>>,
    timeout: Duration,
}

impl TaskExecutor {
    pub fn new(thread_count: usize, timeout_seconds: u64) -> Self {
        let worker_pool = WorkerPool::new(thread_count);
        let running_tasks = Arc::new(Mutex::new(HashMap::new()));
        let timeout = Duration::from_secs(timeout_seconds);

        TaskExecutor {
            worker_pool,
            running_tasks,
            timeout,
        }
    }

    pub fn execute_task(&self, task: Task) -> Result<()> {
        let task_id = task.id;
        let task_arc = Arc::new(task);

        // Mark the task as running
        {
            let mut running = self.running_tasks.lock().unwrap();
            running.insert(task_id, Instant::now());
        }

        // Clone for the closure
        let running_tasks = Arc::clone(&self.running_tasks);

        let job = TaskJob {
            id: task_id,
            task: Arc::clone(&task_arc),
            handler: Box::new(move |task| {
                // Simulate task execution
                println!("Executing task: {}", task.title);
                thread::sleep(Duration::from_secs(2));

                // Mark the task as completed
                {
                    let mut running = running_tasks.lock().unwrap();
                    running.remove(&task_id);
                }

                Ok(())
            }),
        };

        self.worker_pool.execute(job)
    }

    pub fn cancel_task(&self, task_id: u32) -> Result<()> {
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

    pub fn collect_results(&self) -> Vec<JobResult> {
        let mut results = Vec::new();

        while let Some(result) = self.worker_pool.try_get_result() {
            results.push(result);
        }

        results
    }

    pub fn is_task_running(&self, task_id: u32) -> bool {
        let running = self.running_tasks.lock().unwrap();
        running.contains_key(&task_id)
    }
}
