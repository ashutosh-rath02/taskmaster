use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::error::Result;
use crate::project::Project;
use crate::task::Task;

// A simple task cache to avoid repeated loading
pub struct TaskCache {
    projects: HashMap<u32, (Project, Instant)>,
    tasks: HashMap<(u32, u32), (Task, Instant)>,
    ttl: Duration, // Time-to-live for cache entries
}

impl TaskCache {
    pub fn new(ttl_seconds: u64) -> Self {
        TaskCache {
            projects: HashMap::new(),
            tasks: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn add_project(&mut self, project: Project) {
        self.projects.insert(project.id, (project, Instant::now()));
    }

    pub fn get_project(&mut self, id: u32) -> Option<&Project> {
        if let Some((project, timestamp)) = self.projects.get(&id) {
            if timestamp.elapsed() < self.ttl {
                return Some(project);
            }
            // If TTL expired, remove it from cache
            self.projects.remove(&id);
        }
        None
    }

    pub fn add_task(&mut self, project_id: u32, task: Task) {
        self.tasks
            .insert((project_id, task.id), (task, Instant::now()));
    }

    pub fn get_task(&mut self, project_id: u32, task_id: u32) -> Option<&Task> {
        if let Some((task, timestamp)) = self.tasks.get(&(project_id, task_id)) {
            if timestamp.elapsed() < self.ttl {
                return Some(task);
            }
            // If TTL expired, remove it from cache
            self.tasks.remove(&(project_id, task_id));
        }
        None
    }

    pub fn clear(&mut self) {
        self.projects.clear();
        self.tasks.clear();
    }

    pub fn cleanup_expired(&mut self) {
        // Remove expired projects
        let expired_projects: Vec<u32> = self
            .projects
            .iter()
            .filter(|(_, (_, timestamp))| timestamp.elapsed() >= self.ttl)
            .map(|(&id, _)| id)
            .collect();

        for id in expired_projects {
            self.projects.remove(&id);
        }

        // Remove expired tasks
        let expired_tasks: Vec<(u32, u32)> = self
            .tasks
            .iter()
            .filter(|(_, (_, timestamp))| timestamp.elapsed() >= self.ttl)
            .map(|(&key, _)| key)
            .collect();

        for key in expired_tasks {
            self.tasks.remove(&key);
        }
    }
}

// A thread-safe global cache
pub struct GlobalCache {
    inner: Arc<Mutex<TaskCache>>,
}

impl GlobalCache {
    pub fn new(ttl_seconds: u64) -> Self {
        GlobalCache {
            inner: Arc::new(Mutex::new(TaskCache::new(ttl_seconds))),
        }
    }

    pub fn add_project(&self, project: Project) -> Result<()> {
        let mut cache = self.inner.lock().map_err(|_| {
            crate::error::TaskMasterError::InvalidOperation("Cache lock error".to_string())
        })?;
        cache.add_project(project);
        Ok(())
    }

    pub fn get_project(&self, id: u32) -> Result<Option<Project>> {
        let mut cache = self.inner.lock().map_err(|_| {
            crate::error::TaskMasterError::InvalidOperation("Cache lock error".to_string())
        })?;

        Ok(cache.get_project(id).cloned())
    }

    pub fn clear(&self) -> Result<()> {
        let mut cache = self.inner.lock().map_err(|_| {
            crate::error::TaskMasterError::InvalidOperation("Cache lock error".to_string())
        })?;
        cache.clear();
        Ok(())
    }
}
