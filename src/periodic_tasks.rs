use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::task::{Task, TaskPriority, TaskStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecurrencePattern {
    Daily,
    Weekly,
    Monthly,
    Custom(Duration),
}

impl RecurrencePattern {
    pub fn get_next_occurrence(&self, current: SystemTime) -> SystemTime {
        let duration = match self {
            RecurrencePattern::Daily => Duration::from_secs(24 * 60 * 60),
            RecurrencePattern::Weekly => Duration::from_secs(7 * 24 * 60 * 60),
            RecurrencePattern::Monthly => Duration::from_secs(30 * 24 * 60 * 60), // Approximate
            RecurrencePattern::Custom(duration) => *duration,
        };

        current + duration
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodicTask {
    pub id: u32,
    pub template: Task,
    pub pattern: RecurrencePattern,
    pub created_at: SystemTime,
    pub last_run: Option<SystemTime>,
    pub next_run: SystemTime,
    pub occurrences: u32, // How many times this task has been generated
}

impl PeriodicTask {
    pub fn new(id: u32, template: Task, pattern: RecurrencePattern) -> Self {
        let now = SystemTime::now();
        let next_run = pattern.get_next_occurrence(now);

        PeriodicTask {
            id,
            template,
            pattern,
            created_at: now,
            last_run: None,
            next_run,
            occurrences: 0,
        }
    }

    pub fn is_due(&self) -> bool {
        let now = SystemTime::now();
        now >= self.next_run
    }

    pub fn generate_task(&mut self) -> Task {
        let now = SystemTime::now();

        // Update periodic task state
        self.last_run = Some(now);
        self.next_run = self.pattern.get_next_occurrence(now);
        self.occurrences += 1;

        // Create a new task based on the template
        let occurrence_id = self.template.id * 1000 + self.occurrences;
        let title = format!(
            "{} (#{} on {})",
            self.template.title,
            self.occurrences,
            chrono::Local::now().format("%Y-%m-%d"),
        );

        Task::new(
            occurrence_id,
            title,
            self.template.status.clone(),
            self.template.priority.clone(),
        )
    }
}

#[derive(Default)]
pub struct PeriodicTaskScheduler {
    tasks: Vec<PeriodicTask>,
}

impl PeriodicTaskScheduler {
    pub fn new() -> Self {
        PeriodicTaskScheduler { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: PeriodicTask) {
        self.tasks.push(task);
    }

    pub fn remove_task(&mut self, id: u32) -> Option<PeriodicTask> {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
            Some(self.tasks.remove(pos))
        } else {
            None
        }
    }
    pub fn get_all_tasks(&self) -> &[PeriodicTask] {
        &self.tasks
    }

    pub fn get_task_mut(&mut self, id: u32) -> Option<&mut PeriodicTask> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    pub fn get_task(&self, id: u32) -> Option<&PeriodicTask> {
        self.tasks.iter().find(|t| t.id == id)
    }

    pub fn get_due_tasks(&self) -> Vec<&PeriodicTask> {
        self.tasks.iter().filter(|t| t.is_due()).collect()
    }

    pub fn generate_due_tasks(&mut self) -> Vec<Task> {
        let mut generated = Vec::new();

        for task in self.tasks.iter_mut().filter(|t| t.is_due()) {
            generated.push(task.generate_task());
        }

        generated
    }
}
