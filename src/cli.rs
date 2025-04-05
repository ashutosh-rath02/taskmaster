use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::error::Result;
use crate::file_storage::FileStorage;
use crate::project::Project;
use crate::storage::Storage;
use crate::task::{Task, TaskPriority, TaskStatus};

#[derive(Parser)]
#[clap(author, version, about = "TaskMaster - A task management system")]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[clap(long, default_value = "./data", help = "Path to data directory")]
    data_dir: PathBuf,
}

#[derive(ValueEnum, Clone, Debug)]
enum CliTaskStatus {
    Todo,
    InProgress,
    Done,
}

#[derive(ValueEnum, Clone, Debug)]
enum CliTaskPriority {
    Low,
    Medium,
    High,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new project
    CreateProject {
        #[clap(help = "Project ID")]
        id: u32,

        #[clap(help = "Project name")]
        name: String,
    },

    /// List all projects
    ListProjects,

    /// Show project details
    ShowProject {
        #[clap(help = "Project ID")]
        id: u32,
    },

    /// Delete a project
    DeleteProject {
        #[clap(help = "Project ID")]
        id: u32,
    },

    /// Add a task to a project
    AddTask {
        #[clap(help = "Project ID")]
        project_id: u32,

        #[clap(help = "Task ID")]
        id: u32,

        #[clap(help = "Task title")]
        title: String,

        #[clap(value_enum, default_value_t = CliTaskStatus::Todo, help = "Task status")]
        status: CliTaskStatus,

        #[clap(value_enum, default_value_t = CliTaskPriority::Medium, help = "Task priority")]
        priority: CliTaskPriority,
    },

    /// Update a task
    UpdateTask {
        #[clap(help = "Project ID")]
        project_id: u32,

        #[clap(help = "Task ID")]
        id: u32,

        #[clap(help = "New task title")]
        title: String,

        #[clap(value_enum, help = "New task status")]
        status: CliTaskStatus,

        #[clap(value_enum, help = "New task priority")]
        priority: CliTaskPriority,
    },

    /// Delete a task
    DeleteTask {
        #[clap(help = "Project ID")]
        project_id: u32,

        #[clap(help = "Task ID")]
        id: u32,
    },
}

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    let mut storage = FileStorage::new(&cli.data_dir)?;

    match &cli.command {
        Commands::CreateProject { id, name } => {
            let project = Project::new(*id, name.clone());
            storage.save_project(&project)?;
            println!("Project created: {} (ID: {})", name, id);
        }

        Commands::ListProjects => {
            let projects = storage.list_projects()?;
            if projects.is_empty() {
                println!("No projects found");
            } else {
                println!("Projects:");
                for project in projects {
                    println!("  ID: {}, Name: {}", project.id, project.name);
                }
            }
        }

        Commands::ShowProject { id } => match storage.load_project(*id) {
            Ok(project) => {
                println!("Project: {} (ID: {})", project.name, project.id);
                if project.tasks.is_empty() {
                    println!("  No tasks");
                } else {
                    println!("  Tasks:");
                    for task in &project.tasks {
                        println!(
                            "    ID: {}, Title: {}, Status: {:?}, Priority: {:?}",
                            task.id, task.title, task.status, task.priority
                        );
                    }
                }
            }
            Err(e) => println!("Error: {}", e),
        },

        Commands::DeleteProject { id } => match storage.delete_project(*id) {
            Ok(_) => println!("Project deleted: {}", id),
            Err(e) => println!("Error: {}", e),
        },

        Commands::AddTask {
            project_id,
            id,
            title,
            status,
            priority,
        } => {
            // Convert the CLI enums to our internal types
            let task_status = cli_status_to_task_status(status);
            let task_priority = cli_priority_to_task_priority(priority);

            // Create the task
            let task = Task::new(*id, title.clone(), task_status, task_priority);

            // Load the project, add the task, and save it back
            match storage.load_project(*project_id) {
                Ok(mut project) => {
                    project.add_task(task);
                    storage.save_project(&project)?;
                    println!("Task added to project {}: {}", project_id, title);
                }
                Err(e) => println!("Error: {}", e),
            }
        }

        Commands::UpdateTask {
            project_id,
            id,
            title,
            status,
            priority,
        } => {
            // Convert the CLI enums to our internal types
            let task_status = cli_status_to_task_status(status);
            let task_priority = cli_priority_to_task_priority(priority);

            // Load the project, update the task, and save it back
            match storage.load_project(*project_id) {
                Ok(mut project) => {
                    match project.update_task(*id, title.clone(), task_status, task_priority) {
                        Ok(_) => {
                            storage.save_project(&project)?;
                            println!("Task updated: {}", id);
                        }
                        Err(e) => println!("Error updating task: {}", e),
                    }
                }
                Err(e) => println!("Error loading project: {}", e),
            }
        }

        Commands::DeleteTask { project_id, id } => {
            // Load the project, remove the task, and save it back
            match storage.load_project(*project_id) {
                Ok(mut project) => {
                    project.remove_task(*id);
                    storage.save_project(&project)?;
                    println!("Task removed: {}", id);
                }
                Err(e) => println!("Error: {}", e),
            }
        }
    }

    Ok(())
}

// Convert from CLI enums to our internal types
fn cli_status_to_task_status(status: &CliTaskStatus) -> TaskStatus {
    match status {
        CliTaskStatus::Todo => TaskStatus::ToDo,
        CliTaskStatus::InProgress => TaskStatus::InProgress,
        CliTaskStatus::Done => TaskStatus::Done,
    }
}

fn cli_priority_to_task_priority(priority: &CliTaskPriority) -> TaskPriority {
    match priority {
        CliTaskPriority::Low => TaskPriority::Low,
        CliTaskPriority::Medium => TaskPriority::Medium,
        CliTaskPriority::High => TaskPriority::High,
    }
}
