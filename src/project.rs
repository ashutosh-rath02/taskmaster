use serde::{Deserialize, Serialize};

use crate::error::{Result, TaskMasterError};
use crate::task::{Task, TaskPriority, TaskStatus};
use crate::task_dependencies::DependencyGraph;

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

    // Task dependency methods
    pub fn add_task_dependency(&mut self, task_id: u32, dependency_id: u32) -> Result<()> {
        // Check if both tasks exist
        let task_exists = self.tasks.iter().any(|t| t.id == task_id);
        let dependency_exists = self.tasks.iter().any(|t| t.id == dependency_id);

        if !task_exists {
            return Err(TaskMasterError::TaskNotFound(task_id));
        }

        if !dependency_exists {
            return Err(TaskMasterError::TaskNotFound(dependency_id));
        }

        // Use a DependencyGraph to manage dependencies
        let mut graph = DependencyGraph::new();

        // Add existing dependencies
        for task in &self.tasks {
            if let Some(deps) = task.dependencies.as_ref() {
                for &dep_id in deps {
                    graph.add_dependency(task.id, dep_id)?;
                }
            }
        }

        // Add the new dependency
        graph.add_dependency(task_id, dependency_id)?;

        // Update the task's dependencies
        for task in &mut self.tasks {
            if task.id == task_id {
                let deps = graph.get_dependencies(task_id);
                if deps.is_empty() {
                    task.dependencies = None;
                } else {
                    task.dependencies = Some(deps.into_iter().collect());
                }
            }
        }

        Ok(())
    }

    pub fn remove_task_dependency(&mut self, task_id: u32, dependency_id: u32) -> Result<()> {
        // Check if both tasks exist
        let task_exists = self.tasks.iter().any(|t| t.id == task_id);

        if !task_exists {
            return Err(TaskMasterError::TaskNotFound(task_id));
        }

        // Update the task's dependencies
        for task in &mut self.tasks {
            if task.id == task_id {
                if let Some(deps) = task.dependencies.as_mut() {
                    deps.retain(|&id| id != dependency_id);
                    if deps.is_empty() {
                        task.dependencies = None;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_task_execution_order(&self) -> Result<Vec<&Task>> {
        let mut graph = DependencyGraph::new();

        // Add existing dependencies
        for task in &self.tasks {
            if let Some(deps) = task.dependencies.as_ref() {
                for &dep_id in deps {
                    graph.add_dependency(task.id, dep_id)?;
                }
            }
        }

        // Get the execution order as task IDs
        let ordered_ids = graph.get_execution_order(&self.tasks)?;

        // Convert IDs to task references
        let mut ordered_tasks = Vec::new();
        for id in ordered_ids {
            if let Some(task) = self.tasks.iter().find(|t| t.id == id) {
                ordered_tasks.push(task);
            }
        }

        Ok(ordered_tasks)
    }
}
