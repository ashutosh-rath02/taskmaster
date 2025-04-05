mod async_executor;
mod cli;
mod error;
mod file_storage;
mod interactive;
mod notification;
mod project;
mod storage;
mod task;
mod task_executor;
mod task_handler;
mod worker_pool;

use crate::error::{Result, TaskMasterError};
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

    if args.len() > 1 && args[1] == "--test" {
        // Run tests
        run_sync_tests()?;
        test_async().await?;
    } else if args.len() > 1 && args[1] == "--interactive" {
        // Run in interactive mode
        let mut shell = interactive::InteractiveShell::new(&std::path::PathBuf::from("./data"))?;
        shell.run()?;
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
    let (event_tx, event_rx) = mpsc::channel(100);

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
