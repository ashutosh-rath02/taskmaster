use std::any::Any;
use std::fmt::Debug;

use crate::error::Result;
use crate::task::Task;

// A trait that all task handlers must implement
pub trait TaskHandler: Send + Sync + Debug {
    // Execute the task
    fn execute(&self, task: &Task) -> Result<()>;

    // Get the name of the handler
    fn name(&self) -> &str;

    // Check if this handler can process the given task
    fn can_handle(&self, task: &Task) -> bool;

    // Clone the handler (for dynamic dispatch)
    fn clone_box(&self) -> Box<dyn TaskHandler>;

    // Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

// Make TaskHandler objects cloneable
impl Clone for Box<dyn TaskHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// A registry of task handlers
#[derive(Default)]
pub struct TaskHandlerRegistry {
    handlers: Vec<Box<dyn TaskHandler>>,
}

impl TaskHandlerRegistry {
    pub fn new() -> Self {
        TaskHandlerRegistry {
            handlers: Vec::new(),
        }
    }

    pub fn register_handler(&mut self, handler: Box<dyn TaskHandler>) {
        println!("Registering handler: {}", handler.name());
        self.handlers.push(handler);
    }

    pub fn get_handler_for_task(&self, task: &Task) -> Option<&Box<dyn TaskHandler>> {
        self.handlers.iter().find(|h| h.can_handle(task))
    }

    pub fn execute_task(&self, task: &Task) -> Result<()> {
        if let Some(handler) = self.get_handler_for_task(task) {
            println!("Executing task with handler: {}", handler.name());
            handler.execute(task)
        } else {
            Err(crate::error::TaskMasterError::InvalidOperation(format!(
                "No handler available for task: {}",
                task.id
            )))
        }
    }

    pub fn list_handlers(&self) -> Vec<&str> {
        self.handlers.iter().map(|h| h.name()).collect()
    }
}

// Example of a basic task handler implementation
#[derive(Debug, Clone)]
pub struct BasicTaskHandler {
    name: String,
    task_types: Vec<String>,
}

impl BasicTaskHandler {
    pub fn new(name: &str, task_types: Vec<String>) -> Self {
        BasicTaskHandler {
            name: name.to_string(),
            task_types,
        }
    }
}

impl TaskHandler for BasicTaskHandler {
    fn execute(&self, task: &Task) -> Result<()> {
        println!("Basic handler executing task: {}", task.title);
        // Simulate doing something with the task
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, task: &Task) -> bool {
        // For this example, assume task title contains the type
        self.task_types.iter().any(|t| task.title.contains(t))
    }

    fn clone_box(&self) -> Box<dyn TaskHandler> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// A more specialized task handler
#[derive(Debug, Clone)]
pub struct PriorityTaskHandler {
    name: String,
    priority_levels: Vec<crate::task::TaskPriority>,
}

impl PriorityTaskHandler {
    pub fn new(name: &str, priority_levels: Vec<crate::task::TaskPriority>) -> Self {
        PriorityTaskHandler {
            name: name.to_string(),
            priority_levels,
        }
    }
}

impl TaskHandler for PriorityTaskHandler {
    fn execute(&self, task: &Task) -> Result<()> {
        println!(
            "Priority handler executing {:?} priority task: {}",
            task.priority, task.title
        );
        // Prioritized task execution logic would go here
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, task: &Task) -> bool {
        self.priority_levels
            .iter()
            .any(|p| std::mem::discriminant(p) == std::mem::discriminant(&task.priority))
    }

    fn clone_box(&self) -> Box<dyn TaskHandler> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
