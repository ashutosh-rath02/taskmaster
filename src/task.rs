use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TaskStatus {
    ToDo,
    InProgress,
    Done,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub dependencies: Option<Vec<u32>>, // IDs of tasks this task depends on
}

impl Task {
    pub fn new(id: u32, title: String, status: TaskStatus, priority: TaskPriority) -> Self {
        Task {
            id,
            title,
            status,
            priority,
            dependencies: None,
        }
    }

    pub fn update(
        &mut self,
        new_title: String,
        new_status: TaskStatus,
        new_priority: TaskPriority,
    ) {
        self.title = new_title;
        self.status = new_status;
        self.priority = new_priority;
    }

    pub fn display(&self) {
        println!(
            "Task ID: {}, Title: {}, Status: {:?}, Priority: {:?}",
            self.id, self.title, self.status, self.priority
        );

        if let Some(deps) = &self.dependencies {
            if !deps.is_empty() {
                println!("  Dependencies: {:?}", deps);
            }
        }
    }

    // Add a method to check if this task can be started
    pub fn can_start(&self, tasks: &[Task]) -> bool {
        if let Some(deps) = &self.dependencies {
            if deps.is_empty() {
                return true;
            }

            // Create a map for quick lookup
            let task_map: HashMap<u32, &Task> = tasks.iter().map(|t| (t.id, t)).collect();

            // Check each dependency
            for &dep_id in deps {
                if let Some(&dep_task) = task_map.get(&dep_id) {
                    if let TaskStatus::Done = dep_task.status {
                        // This dependency is met
                    } else {
                        // This dependency is not met
                        return false;
                    }
                } else {
                    // Dependency task not found, consider it not met
                    return false;
                }
            }

            true
        } else {
            // No dependencies, can start immediately
            true
        }
    }
}

pub struct TaskBuilder {
    id: u32,
    title: String,
    status: Option<TaskStatus>,
    priority: Option<TaskPriority>,
    dependencies: Option<Vec<u32>>,
}

impl TaskBuilder {
    pub fn new(id: u32, title: String) -> Self {
        TaskBuilder {
            id,
            title,
            status: None,
            priority: None,
            dependencies: None,
        }
    }

    pub fn status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn dependency(mut self, dependency_id: u32) -> Self {
        let deps = self.dependencies.get_or_insert(Vec::new());
        deps.push(dependency_id);
        self
    }

    pub fn build(self) -> Task {
        Task {
            id: self.id,
            title: self.title,
            status: self.status.unwrap_or(TaskStatus::ToDo),
            priority: self.priority.unwrap_or(TaskPriority::Medium),
            dependencies: if let Some(deps) = self.dependencies {
                if deps.is_empty() {
                    None
                } else {
                    Some(deps)
                }
            } else {
                None
            },
        }
    }
}
