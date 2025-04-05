use serde::{Deserialize, Serialize};

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
}

impl Task {
    pub fn new(id: u32, title: String, status: TaskStatus, priority: TaskPriority) -> Self {
        Task {
            id,
            title,
            status,
            priority,
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
    }
}

pub struct TaskBuilder {
    id: u32,
    title: String,
    status: Option<TaskStatus>,
    priority: Option<TaskPriority>,
}

impl TaskBuilder {
    pub fn new(id: u32, title: String) -> Self {
        TaskBuilder {
            id,
            title,
            status: None,
            priority: None,
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

    pub fn build(self) -> Task {
        Task {
            id: self.id,
            title: self.title,
            status: self.status.unwrap_or(TaskStatus::ToDo),
            priority: self.priority.unwrap_or(TaskPriority::Medium),
        }
    }
}
