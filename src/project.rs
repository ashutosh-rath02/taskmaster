use serde::{Deserialize, Serialize};

use crate::error::{Result, TaskMasterError};
use crate::task::{Task, TaskPriority, TaskStatus};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub tasks: Vec<Task>,
}

impl Project {
    pub fn new(id: u32, name: String) -> Self {
        Project {
            id,
            name,
            tasks: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn remove_task(&mut self, task_id: u32) {
        self.tasks.retain(|task| task.id != task_id);
    }

    pub fn update_task(
        &mut self,
        task_id: u32,
        new_title: String,
        new_status: TaskStatus,
        new_priority: TaskPriority,
    ) -> Result<()> {
        let task = self
            .tasks
            .iter_mut()
            .find(|task| task.id == task_id)
            .ok_or(TaskMasterError::TaskNotFound(task_id))?;

        task.title = new_title;
        task.status = new_status;
        task.priority = new_priority;
        Ok(())
    }

    pub fn get_task(&self, task_id: u32) -> Result<&Task> {
        self.tasks
            .iter()
            .find(|task| task.id == task_id)
            .ok_or(TaskMasterError::TaskNotFound(task_id))
    }

    pub fn display(&self) {
        println!("Project ID: {}, Name: {}", self.id, self.name);
        println!("Tasks:");
        for task in &self.tasks {
            task.display();
        }
    }
}
