# TaskMaster

A comprehensive task management system built in Rust, showcasing advanced Rust concepts including ownership, concurrency, async programming, and trait-based polymorphism.

## Features

- **Core Task Management**: Create, read, update, and delete tasks and projects
- **Storage**: File-based JSON storage with proper error handling
- **Concurrency**: Worker pool for parallel task processing
- **Async Programming**: Non-blocking execution with Tokio
- **Task Dependencies**: Define dependencies between tasks with cycle detection
- **Periodic Tasks**: Schedule recurring tasks with custom intervals
- **Multiple Interfaces**:
  - Command Line Interface (CLI)
  - Interactive Shell
  - Terminal User Interface (TUI)
- **Performance Optimization**: Caching system for improved response time

## Installation

Make sure you have Rust and Cargo installed. If not, install them from [rustup.rs](https://rustup.rs/).

```bash
# Clone the repository
git clone https://github.com/yourusername/taskmaster.git
cd taskmaster

# Build the project
cargo build --release

# Run the application
cargo run --release
```

## How to Use

TaskMaster offers three different interfaces to interact with your tasks and projects:

### Terminal User Interface (TUI)

The TUI provides a user-friendly interface for managing your tasks and projects:

```bash
cargo run -- --tui
```

#### Navigation:

- Use **Tab** to switch between Projects, Tasks, and Help tabs
- Use **Up/Down** arrow keys to navigate through lists
- Press **Enter** to select a project and view its tasks

#### Adding Items:

- Press **a** to add a new project or task
- For projects, use the format: `ID Name` (e.g., `1 My Project`)
- For tasks, use the format: `ID Title` (e.g., `1 My Task`)
- Press **Enter** to confirm or **Esc** to cancel

#### Deleting Items:

- Select an item with arrow keys
- Press **d** to delete the selected item

#### Exiting:

- Press **q** to quit the application

### Command Line Interface (CLI)

The CLI is the default mode and allows you to execute specific commands:

```bash
cargo run -- <command> [arguments]
```

Available commands:

- `create-project <id> <name>`: Create a new project
- `list-projects`: List all projects
- `show-project <id>`: Show details of a specific project
- `delete-project <id>`: Delete a project
- `add-task <project_id> <id> <title> <status> <priority>`: Add a task to a project
- `update-task <project_id> <id> <title> <status> <priority>`: Update a task
- `delete-task <project_id> <id>`: Delete a task

Examples:

```bash
# Create a new project
cargo run -- create-project 1 "My First Project"

# List all projects
cargo run -- list-projects

# Add a task to project 1
cargo run -- add-task 1 1 "Important Task" Todo High
```

### Running Tests

To run the test suite:

```bash
cargo run -- --test
```

## Project Structure

- **Core Data Structures**: Task and Project structures with associated operations
- **Storage Layer**: JSON-based file storage system
- **Concurrency**: Worker pool for parallel task execution
- **Async Runtime**: Tokio-based async task execution
- **Dependency Management**: Directed graph implementation for task dependencies
- **Periodic Tasks**: Scheduler for recurring tasks
- **User Interfaces**: CLI, Interactive Shell, and TUI implementations

## Implementation Details

### Phase 1: Foundation

- Core data structures and operations
- Error handling with custom error types
- File-based storage with serde serialization
- Ownership and borrowing patterns

### Phase 2: Advanced Features

- Concurrency with worker pools and channels
- Async programming with Tokio
- Advanced type features with trait objects
- Extensible plugin system

### Phase 3: User Interface and Extensions

- Command-line interface with clap
- Interactive shell mode
- Terminal UI with crossterm and tui
- Task dependencies and scheduling

## Dependencies

- `serde`: Serialization/deserialization
- `serde_json`: JSON format support
- `tokio`: Async runtime
- `futures`: Async utilities
- `clap`: Command-line argument parsing
- `crossterm`: Terminal handling
- `tui` (ratatui): Terminal user interface
- `chrono`: Date and time handling

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
