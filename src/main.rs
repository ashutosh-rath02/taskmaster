mod async_executor;
mod cli;
mod error;
mod file_storage;
mod interactive;
mod notification;
mod periodic_tasks;
mod project;
mod storage;
mod task;
mod task_dependencies;
mod task_executor;
mod task_handler;
mod tui;
mod worker_pool;

use crate::error::Result;
use crate::file_storage::FileStorage;
use crate::project::Project;
use crate::storage::Storage;
use crate::task::{Task, TaskBuilder, TaskPriority, TaskStatus};

use tokio::sync::mpsc;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Check if we're running tests or in normal mode
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--test" => {
                // Run tests
                run_sync_tests()?;
                test_async().await?;
            }
            "--interactive" => {
                // Run in interactive mode
                let mut shell =
                    interactive::InteractiveShell::new(&std::path::PathBuf::from("./data"))?;
                shell.run()?;
            }
            "--tui" => {
                // Run with Terminal UI
                tui::run_tui()?;
            }
            _ => {
                // Run in CLI mode
                cli::run_cli()?;
            }
        }
    } else {
        // Run in CLI mode
        cli::run_cli()?;
    }

    Ok(())
}

fn run_sync_tests() -> Result<()> {
    // Basic project demonstration
    let mut project = Project::new(1, String::from("Project 1"));

    let task1 = Task::new(
        1,
        String::from("Task 1"),
        TaskStatus::ToDo,
        TaskPriority::High,
    );
    let task2 = Task::new(
        2,
        String::from("Task 2"),
        TaskStatus::InProgress,
        TaskPriority::Medium,
    );

    project.add_task(task1);
    project.add_task(task2);

    println!("Initial project:");
    project.display();

    println!("\nTesting storage functionality:");
    if let Err(e) = test_storage() {
        println!("Storage test failed: {}", e);
    }

    println!("\nTesting concurrency:");
    if let Err(e) = test_concurrency() {
        println!("Concurrency test failed: {}", e);
    }

    println!("\nTesting advanced type features:");
    if let Err(e) = test_advanced_types() {
        println!("Advanced type features test failed: {}", e);
    }

    println!("\nTesting task dependencies:");
    if let Err(e) = test_task_dependencies() {
        println!("Task dependencies test failed: {}", e);
    }

    println!("\nTesting periodic tasks:");
    if let Err(e) = test_periodic_tasks() {
        println!("Periodic tasks test failed: {}", e);
    }

    Ok(())
}

fn test_storage() -> Result<()> {
    // Create a test project with tasks
    let mut project = Project::new(42, String::from("Test Project"));

    let task1 = Task::new(
        101,
        String::from("Implement storage"),
        TaskStatus::Done,
        TaskPriority::High,
    );

    // Use the builder pattern for the second task
    let task2 = TaskBuilder::new(102, String::from("Test storage"))
        .status(TaskStatus::InProgress)
        .priority(TaskPriority::Medium)
        .build();

    project.add_task(task1);
    project.add_task(task2);

    // Initialize storage
    let mut storage = FileStorage::new("./data")?;

    // Save project
    println!("Saving project...");
    storage.save_project(&project)?;

    // List all projects
    println!("Listing all projects:");
    let projects = storage.list_projects()?;
    println!("Found {} projects", projects.len());

    // Load project
    println!("Loading project...");
    let loaded_project = storage.load_project(42)?;

    // Verify project
    println!("Verifying project...");
    assert_eq!(loaded_project.id, project.id);
    assert_eq!(loaded_project.name, project.name);
    assert_eq!(loaded_project.tasks.len(), project.tasks.len());

    // Display loaded project
    println!("Loaded project:");
    loaded_project.display();

    // Test non-existent project
    println!("Testing error handling with non-existent project...");
    match storage.load_project(999) {
        Ok(_) => println!("Unexpectedly found project 999"),
        Err(e) => println!("As expected, error: {}", e),
    }

    println!("Storage test passed!");
    Ok(())
}

fn test_concurrency() -> Result<()> {
    use crate::task_executor::TaskExecutor;

    println!("Testing concurrency...");

    // Create a task executor with 4 worker threads and 10-second timeout
    let executor = TaskExecutor::new(4, 10);

    // Create some tasks
    let task1 = Task::new(
        1,
        String::from("Task 1"),
        TaskStatus::ToDo,
        TaskPriority::High,
    );
    let task2 = Task::new(
        2,
        String::from("Task 2"),
        TaskStatus::ToDo,
        TaskPriority::Medium,
    );
    let task3 = Task::new(
        3,
        String::from("Task 3"),
        TaskStatus::ToDo,
        TaskPriority::Low,
    );

    // Execute tasks
    executor.execute_task(task1)?;
    executor.execute_task(task2)?;
    executor.execute_task(task3)?;

    // Check if tasks are running
    println!("Is task 1 running? {}", executor.is_task_running(1));
    println!("Is task 2 running? {}", executor.is_task_running(2));
    println!("Is task 3 running? {}", executor.is_task_running(3));

    // Wait a bit for some tasks to complete
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Collect and display results
    let results = executor.collect_results();
    println!("Collected {} results", results.len());
    for result in &results {
        println!(
            "Task {}: {}",
            result.task_id,
            if result.success { "Success" } else { "Failed" }
        );
    }

    // Check for timeouts
    let timed_out = executor.check_timeouts();
    println!("Timed out tasks: {:?}", timed_out);

    println!("Concurrency test completed");
    Ok(())
}

fn test_advanced_types() -> Result<()> {
    use crate::task_handler::{BasicTaskHandler, PriorityTaskHandler, TaskHandlerRegistry};

    println!("\nTesting advanced type features:");

    // Create task handlers
    let basic_handler = BasicTaskHandler::new(
        "BasicHandler",
        vec!["Report".to_string(), "Document".to_string()],
    );

    let priority_handler = PriorityTaskHandler::new(
        "PriorityHandler",
        vec![TaskPriority::High, TaskPriority::Medium],
    );

    // Create registry and register handlers
    let mut registry = TaskHandlerRegistry::new();
    registry.register_handler(Box::new(basic_handler));
    registry.register_handler(Box::new(priority_handler));

    // List available handlers
    println!("Available handlers: {:?}", registry.list_handlers());

    // Create tasks to test with different handlers
    let report_task = Task::new(
        101,
        String::from("Generate Monthly Report"),
        TaskStatus::ToDo,
        TaskPriority::Low,
    );

    let urgent_task = Task::new(
        102,
        String::from("Fix Production Bug"),
        TaskStatus::ToDo,
        TaskPriority::High,
    );

    // Execute tasks with appropriate handlers
    println!("Executing report task:");
    registry.execute_task(&report_task)?;

    println!("Executing urgent task:");
    registry.execute_task(&urgent_task)?;

    // Try a task that no handler can process
    let unhandled_task = Task::new(
        103,
        String::from("Unknown Task Type"),
        TaskStatus::ToDo,
        TaskPriority::Low,
    );

    println!("Trying to execute unhandled task:");
    match registry.execute_task(&unhandled_task) {
        Ok(_) => println!("Task executed successfully"),
        Err(e) => println!("Expected error: {}", e),
    }

    println!("Advanced type features test completed");
    Ok(())
}

async fn test_async() -> Result<()> {
    use crate::async_executor::{AsyncTaskExecutor, TaskEvent};
    use crate::notification::NotificationSystem;

    println!("\nTesting async features:");

    // Create channels for notifications
    let (_event_tx, event_rx) = mpsc::channel(100);

    // Create async executor
    let executor = AsyncTaskExecutor::new(10, 100);

    // Create notification system
    let mut notification_system = NotificationSystem::new(event_rx);

    // Register callbacks
    notification_system.register_callback("log_events", |event| match event {
        TaskEvent::Started { task_id } => println!("NOTIFICATION: Task {} started", task_id),
        TaskEvent::Completed { task_id } => println!("NOTIFICATION: Task {} completed", task_id),
        TaskEvent::Failed {
            task_id,
            error_message,
        } => {
            println!("NOTIFICATION: Task {} failed: {}", task_id, error_message)
        }
        TaskEvent::Timeout { task_id } => println!("NOTIFICATION: Task {} timed out", task_id),
        TaskEvent::Terminated { task_id } => println!("NOTIFICATION: Task {} terminated", task_id),
    });
    // Start notification system in background
    tokio::spawn(async move {
        notification_system.start().await.unwrap();
    });

    // Create some tasks
    let task1 = Task::new(
        1,
        String::from("Async Task 1"),
        TaskStatus::ToDo,
        TaskPriority::High,
    );
    let task2 = Task::new(
        2,
        String::from("Async Task 2"),
        TaskStatus::ToDo,
        TaskPriority::Medium,
    );

    // Execute tasks
    executor.execute_task(task1).await?;
    executor.execute_task(task2).await?;

    // Wait to see the results
    time::sleep(Duration::from_secs(5)).await;

    println!("Async test completed");
    Ok(())
}

fn test_task_dependencies() -> Result<()> {
    println!("\nTesting task dependencies:");

    // Create a project with interdependent tasks
    let mut project = Project::new(99, String::from("Dependencies Test"));

    // Task chain: task1 <- task2 <- task3 <- task4
    let task1 = Task::new(
        1,
        String::from("Requirement gathering"),
        TaskStatus::ToDo,
        TaskPriority::High,
    );
    let task2 = Task::new(
        2,
        String::from("Design"),
        TaskStatus::ToDo,
        TaskPriority::High,
    );
    let task3 = Task::new(
        3,
        String::from("Implementation"),
        TaskStatus::ToDo,
        TaskPriority::Medium,
    );
    let task4 = Task::new(
        4,
        String::from("Testing"),
        TaskStatus::ToDo,
        TaskPriority::Medium,
    );

    project.add_task(task1);
    project.add_task(task2);
    project.add_task(task3);
    project.add_task(task4);

    // Add dependencies
    project.add_task_dependency(2, 1)?; // task2 depends on task1
    project.add_task_dependency(3, 2)?; // task3 depends on task2
    project.add_task_dependency(4, 3)?; // task4 depends on task3

    // Get the execution order
    println!("Task execution order:");
    let order = project.get_task_execution_order()?;
    for (idx, task) in order.iter().enumerate() {
        println!("  {}: {} (ID: {})", idx + 1, task.title, task.id);
    }

    // Try adding a circular dependency (should fail)
    println!("\nTrying to create a circular dependency:");
    match project.add_task_dependency(1, 4) {
        Ok(_) => println!("Unexpected success: circular dependency was added"),
        Err(e) => println!("Expected error: {}", e),
    }

    // Check if tasks can be started
    println!("\nChecking which tasks can be started:");
    for task in &project.tasks {
        println!(
            "  Task {}: {} - Can start: {}",
            task.id,
            task.title,
            task.can_start(&project.tasks)
        );
    }

    // Mark task1 as done and check again
    println!("\nMarking task 1 as done and checking again:");
    if let Some(task) = project.tasks.iter_mut().find(|t| t.id == 1) {
        task.status = TaskStatus::Done;
    }

    for task in &project.tasks {
        println!(
            "  Task {}: {} - Can start: {}",
            task.id,
            task.title,
            task.can_start(&project.tasks)
        );
    }

    println!("Task dependencies test completed");
    Ok(())
}

fn test_periodic_tasks() -> Result<()> {
    use crate::periodic_tasks::{PeriodicTask, PeriodicTaskScheduler, RecurrencePattern};
    println!("\nTesting periodic tasks:");

    // Create a task template
    let template_task = Task::new(
        100,
        String::from("Weekly Report"),
        TaskStatus::ToDo,
        TaskPriority::Medium,
    );

    // Create a periodic task with a weekly pattern
    let periodic_task = PeriodicTask::new(1, template_task.clone(), RecurrencePattern::Weekly);

    println!(
        "Created periodic task: {} (ID: {})",
        periodic_task.template.title, periodic_task.id
    );

    // Create a scheduler
    let mut scheduler = PeriodicTaskScheduler::new();

    // Add the periodic task
    scheduler.add_task(periodic_task);

    // Create a daily standup task
    let standup_template = Task::new(
        200,
        String::from("Daily Standup"),
        TaskStatus::ToDo,
        TaskPriority::High,
    );

    let standup_task = PeriodicTask::new(2, standup_template, RecurrencePattern::Daily);

    scheduler.add_task(standup_task);

    // Create a custom interval task
    let backup_template = Task::new(
        300,
        String::from("System Backup"),
        TaskStatus::ToDo,
        TaskPriority::Low,
    );

    let backup_task = PeriodicTask::new(
        3,
        backup_template,
        RecurrencePattern::Custom(Duration::from_secs(12 * 60 * 60)), // Every 12 hours
    );

    scheduler.add_task(backup_task);

    // Normally we'd wait for tasks to become due, but for testing,
    // we'll force all tasks to be "due" by modifying next_run to be in the past
    for id in 1..=3 {
        // Assuming task IDs 1, 2, 3
        if let Some(task) = scheduler.get_task_mut(id) {
            task.next_run = std::time::SystemTime::now() - Duration::from_secs(1);
        }
    }

    // Generate due tasks
    println!("Generating due tasks:");
    let generated = scheduler.generate_due_tasks();

    // Display generated tasks
    for task in &generated {
        println!("  Generated: {} (ID: {})", task.title, task.id);
    }

    // Check that the next_run has been updated
    println!("\nNext scheduled runs:");
    for task in scheduler.get_all_tasks() {
        println!(
            "  Task {}: next run in future: {}",
            task.id,
            task.next_run > std::time::SystemTime::now()
        );
    }

    println!("Periodic tasks test completed");
    Ok(())
}
