use std::io::{self, Write};
use std::path::PathBuf;

use crate::error::Result;
use crate::file_storage::FileStorage;
use crate::project::Project;
use crate::storage::Storage;
use crate::task::{Task, TaskPriority, TaskStatus};

pub struct InteractiveShell {
    storage: FileStorage,
    current_project: Option<Project>,
}

impl InteractiveShell {
    pub fn new(data_dir: &PathBuf) -> Result<Self> {
        let storage = FileStorage::new(data_dir)?;
        Ok(InteractiveShell {
            storage,
            current_project: None,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        println!("TaskMaster Interactive Shell");
        println!("Type 'help' for a list of commands");

        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            let command = parts[0];

            match command {
                "help" => self.show_help(),
                "exit" | "quit" => break,
                "list" => self.list_projects()?,
                "new" if parts.len() >= 3 => {
                    let id = parts[1].parse::<u32>().unwrap_or(0);
                    let name = parts[2..].join(" ");
                    self.create_project(id, &name)?;
                }
                "open" if parts.len() >= 2 => {
                    let id = parts[1].parse::<u32>().unwrap_or(0);
                    self.open_project(id)?;
                }
                "delete" if parts.len() >= 2 => {
                    let id = parts[1].parse::<u32>().unwrap_or(0);
                    self.delete_project(id)?;
                }
                "tasks" => self.list_tasks()?,
                "add" if parts.len() >= 3 => {
                    let id = parts[1].parse::<u32>().unwrap_or(0);
                    let title = parts[2..].join(" ");
                    self.add_task(id, &title)?;
                }
                "update" if parts.len() >= 5 => {
                    let id = parts[1].parse::<u32>().unwrap_or(0);
                    let title = parts[2].to_string();
                    let status = parts[3].to_string();
                    let priority = parts[4].to_string();
                    self.update_task(id, &title, &status, &priority)?;
                }
                "remove" if parts.len() >= 2 => {
                    let id = parts[1].parse::<u32>().unwrap_or(0);
                    self.remove_task(id)?;
                }
                _ => println!("Unknown command or invalid format. Type 'help' for help."),
            }
        }

        println!("Goodbye!");
        Ok(())
    }

    fn show_help(&self) {
        println!("Available commands:");
        println!("  help                          Show this help message");
        println!("  exit, quit                    Exit the shell");
        println!("  list                          List all projects");
        println!("  new <id> <name>               Create a new project");
        println!("  open <id>                     Open a project (make it current)");
        println!("  delete <id>                   Delete a project");
        println!("  tasks                         List tasks in the current project");
        println!("  add <id> <title>              Add a task to the current project");
        println!("  update <id> <title> <status> <priority>  Update a task");
        println!("  remove <id>                   Remove a task from the current project");
    }

    fn list_projects(&self) -> Result<()> {
        let projects = self.storage.list_projects()?;
        if projects.is_empty() {
            println!("No projects found");
        } else {
            println!("Projects:");
            for project in projects {
                println!("  ID: {}, Name: {}", project.id, project.name);
            }
        }
        Ok(())
    }

    fn create_project(&mut self, id: u32, name: &str) -> Result<()> {
        let project = Project::new(id, name.to_string());
        self.storage.save_project(&project)?;
        println!("Project created: {} (ID: {})", name, id);
        Ok(())
    }

    fn open_project(&mut self, id: u32) -> Result<()> {
        match self.storage.load_project(id) {
            Ok(project) => {
                println!("Opened project: {} (ID: {})", project.name, project.id);
                self.current_project = Some(project);
                Ok(())
            }
            Err(e) => {
                println!("Error opening project: {}", e);
                Ok(())
            }
        }
    }

    fn delete_project(&mut self, id: u32) -> Result<()> {
        // If the project to delete is the current project, clear it
        if let Some(proj) = &self.current_project {
            if proj.id == id {
                self.current_project = None;
            }
        }

        match self.storage.delete_project(id) {
            Ok(_) => {
                println!("Project deleted: {}", id);
                Ok(())
            }
            Err(e) => {
                println!("Error deleting project: {}", e);
                Ok(())
            }
        }
    }

    fn list_tasks(&self) -> Result<()> {
        if let Some(project) = &self.current_project {
            if project.tasks.is_empty() {
                println!("No tasks in project");
            } else {
                println!("Tasks in project {}:", project.name);
                for task in &project.tasks {
                    println!(
                        "  ID: {}, Title: {}, Status: {:?}, Priority: {:?}",
                        task.id, task.title, task.status, task.priority
                    );
                }
            }
        } else {
            println!("No project is currently open");
        }
        Ok(())
    }

    fn add_task(&mut self, id: u32, title: &str) -> Result<()> {
        if let Some(project) = &mut self.current_project {
            let task = Task::new(
                id,
                title.to_string(),
                TaskStatus::ToDo,
                TaskPriority::Medium,
            );
            project.add_task(task);
            self.storage.save_project(project)?;
            println!("Task added: {} (ID: {})", title, id);
        } else {
            println!("No project is currently open");
        }
        Ok(())
    }

    fn update_task(&mut self, id: u32, title: &str, status: &str, priority: &str) -> Result<()> {
        if let Some(project) = &mut self.current_project {
            let status = match status.to_lowercase().as_str() {
                "todo" => TaskStatus::ToDo,
                "in_progress" | "inprogress" => TaskStatus::InProgress,
                "done" => TaskStatus::Done,
                _ => {
                    println!("Invalid status: {}", status);
                    return Ok(());
                }
            };

            let priority = match priority.to_lowercase().as_str() {
                "low" => TaskPriority::Low,
                "medium" => TaskPriority::Medium,
                "high" => TaskPriority::High,
                _ => {
                    println!("Invalid priority: {}", priority);
                    return Ok(());
                }
            };

            match project.update_task(id, title.to_string(), status, priority) {
                Ok(_) => {
                    self.storage.save_project(project)?;
                    println!("Task updated: {}", id);
                }
                Err(e) => println!("Error updating task: {}", e),
            }
        } else {
            println!("No project is currently open");
        }
        Ok(())
    }

    fn remove_task(&mut self, id: u32) -> Result<()> {
        if let Some(project) = &mut self.current_project {
            project.remove_task(id);
            self.storage.save_project(project)?;
            println!("Task removed: {}", id);
        } else {
            println!("No project is currently open");
        }
        Ok(())
    }
}
