use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Terminal,
};

use crate::error::Result;
use crate::file_storage::FileStorage;
use crate::project::Project;
use crate::storage::Storage;
use crate::task::{Task, TaskPriority, TaskStatus};

enum InputMode {
    Normal,
    Editing,
}

enum AppTab {
    Projects,
    Tasks,
    Help,
}

struct App {
    tabs: Vec<&'static str>,
    active_tab: AppTab,
    projects: Vec<Project>,
    projects_state: ListState,
    tasks: Vec<Task>,
    tasks_state: ListState,
    input_mode: InputMode,
    input: String,
    storage: FileStorage,
    status_message: String,
}

impl App {
    fn new() -> Result<Self> {
        // Initialize with data directory
        let storage = FileStorage::new("./data")?;

        // Load projects
        let projects = storage.list_projects()?;

        // Initialize list states
        let mut projects_state = ListState::default();
        let mut tasks_state = ListState::default();

        // If there are projects, select the first one
        if !projects.is_empty() {
            projects_state.select(Some(0));
        }

        Ok(App {
            tabs: vec!["Projects", "Tasks", "Help"],
            active_tab: AppTab::Projects,
            projects,
            projects_state,
            tasks: Vec::new(),
            tasks_state,
            input_mode: InputMode::Normal,
            input: String::new(),
            storage,
            status_message: String::new(),
        })
    }

    fn load_project_tasks(&mut self) -> Result<()> {
        // If a project is selected, load its tasks
        if let Some(index) = self.projects_state.selected() {
            if let Some(project) = self.projects.get(index) {
                // Load the project to get its tasks
                match self.storage.load_project(project.id) {
                    Ok(loaded_project) => {
                        self.tasks = loaded_project.tasks;
                        // Reset task selection
                        if !self.tasks.is_empty() {
                            self.tasks_state.select(Some(0));
                        } else {
                            self.tasks_state.select(None);
                        }
                    }
                    Err(e) => {
                        self.status_message = format!("Error loading tasks: {}", e);
                        self.tasks.clear();
                        self.tasks_state.select(None);
                    }
                }
            }
        } else {
            self.tasks.clear();
            self.tasks_state.select(None);
        }
        Ok(())
    }

    fn add_project(&mut self) -> Result<()> {
        // Parse the input as "ID Name"
        let parts: Vec<&str> = self.input.trim().splitn(2, ' ').collect();
        if parts.len() < 2 {
            self.status_message = "Invalid format. Use: ID Name".to_string();
            return Ok(());
        }

        let id = match parts[0].parse::<u32>() {
            Ok(id) => id,
            Err(_) => {
                self.status_message = "Invalid ID. Use a number.".to_string();
                return Ok(());
            }
        };

        let name = parts[1].to_string();

        // Create and save the project
        let project = Project::new(id, name);
        self.storage.save_project(&project)?;

        // Refresh projects list
        self.projects = self.storage.list_projects()?;
        self.status_message = "Project added successfully.".to_string();

        // Clear input
        self.input.clear();

        // Select the new project if it's the first one
        if self.projects.len() == 1 {
            self.projects_state.select(Some(0));
        }

        Ok(())
    }

    fn add_task(&mut self) -> Result<()> {
        // Ensure a project is selected
        if let Some(project_index) = self.projects_state.selected() {
            if let Some(project) = self.projects.get_mut(project_index) {
                // Parse the input as "ID Title"
                let parts: Vec<&str> = self.input.trim().splitn(2, ' ').collect();
                if parts.len() < 2 {
                    self.status_message = "Invalid format. Use: ID Title".to_string();
                    return Ok(());
                }

                let id = match parts[0].parse::<u32>() {
                    Ok(id) => id,
                    Err(_) => {
                        self.status_message = "Invalid ID. Use a number.".to_string();
                        return Ok(());
                    }
                };

                let title = parts[1].to_string();

                // Create the task
                let task = Task::new(id, title, TaskStatus::ToDo, TaskPriority::Medium);

                // Load the full project, add the task, and save
                match self.storage.load_project(project.id) {
                    Ok(mut loaded_project) => {
                        loaded_project.add_task(task);
                        self.storage.save_project(&loaded_project)?;
                        self.status_message = "Task added successfully.".to_string();

                        // Reload tasks
                        self.load_project_tasks()?;
                    }
                    Err(e) => {
                        self.status_message = format!("Error loading project: {}", e);
                    }
                }

                // Clear input
                self.input.clear();
            }
        } else {
            self.status_message = "Please select a project first.".to_string();
        }

        Ok(())
    }

    // Move selection up in the current list
    fn select_previous(&mut self) {
        match self.active_tab {
            AppTab::Projects => {
                let i = match self.projects_state.selected() {
                    Some(i) => {
                        if i > 0 {
                            i - 1
                        } else {
                            i
                        }
                    }
                    None => 0,
                };
                self.projects_state.select(Some(i));
            }
            AppTab::Tasks => {
                let i = match self.tasks_state.selected() {
                    Some(i) => {
                        if i > 0 {
                            i - 1
                        } else {
                            i
                        }
                    }
                    None => 0,
                };
                self.tasks_state.select(Some(i));
            }
            _ => {}
        }
    }

    // Move selection down in the current list
    fn select_next(&mut self) {
        match self.active_tab {
            AppTab::Projects => {
                let i = match self.projects_state.selected() {
                    Some(i) => {
                        if i < self.projects.len().saturating_sub(1) {
                            i + 1
                        } else {
                            i
                        }
                    }
                    None => 0,
                };
                self.projects_state.select(Some(i));
            }
            AppTab::Tasks => {
                let i = match self.tasks_state.selected() {
                    Some(i) => {
                        if i < self.tasks.len().saturating_sub(1) {
                            i + 1
                        } else {
                            i
                        }
                    }
                    None => 0,
                };
                self.tasks_state.select(Some(i));
            }
            _ => {}
        }
    }
}

pub fn run_tui() -> Result<()> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new()?;

    // Main loop
    loop {
        // Draw the UI
        terminal.draw(|f| {
            let size = f.size();

            // Create layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(1),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            // Create tabs
            let tabs_vec: Vec<Spans> = app
                .tabs
                .iter()
                .map(|t| Spans::from(Span::raw(*t)))
                .collect();
            let tabs = Tabs::new(tabs_vec)
                .block(Block::default().borders(Borders::ALL).title("Tabs"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(match app.active_tab {
                    AppTab::Projects => 0,
                    AppTab::Tasks => 1,
                    AppTab::Help => 2,
                })
                .divider("|");

            f.render_widget(tabs, chunks[0]);

            // Render content based on active tab
            match app.active_tab {
                AppTab::Projects => {
                    // Project list
                    let project_items: Vec<ListItem> = app
                        .projects
                        .iter()
                        .map(|p| {
                            ListItem::new(Spans::from(Span::raw(format!(
                                "ID: {} - {}",
                                p.id, p.name
                            ))))
                        })
                        .collect();

                    let projects = List::new(project_items)
                        .block(Block::default().borders(Borders::ALL).title("Projects"))
                        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                        .highlight_symbol("> ");

                    f.render_stateful_widget(projects, chunks[1], &mut app.projects_state);
                }
                AppTab::Tasks => {
                    // Task list
                    let task_items: Vec<ListItem> = app
                        .tasks
                        .iter()
                        .map(|t| {
                            ListItem::new(Spans::from(Span::raw(format!(
                                "ID: {} - {} [Status: {:?}, Priority: {:?}]",
                                t.id, t.title, t.status, t.priority
                            ))))
                        })
                        .collect();

                    let tasks = List::new(task_items)
                        .block(Block::default().borders(Borders::ALL).title("Tasks"))
                        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                        .highlight_symbol("> ");

                    f.render_stateful_widget(tasks, chunks[1], &mut app.tasks_state);
                }
                AppTab::Help => {
                    let help_text = vec![
                        Spans::from(Span::raw("Navigation:")),
                        Spans::from(Span::raw("  Tab - Switch between tabs")),
                        Spans::from(Span::raw("  Up/Down - Navigate list")),
                        Spans::from(Span::raw("  Enter - Select project/task")),
                        Spans::from(Span::raw("")),
                        Spans::from(Span::raw("Commands:")),
                        Spans::from(Span::raw("  a - Add a project/task")),
                        Spans::from(Span::raw("  d - Delete selected item")),
                        Spans::from(Span::raw("  q - Quit")),
                        Spans::from(Span::raw("")),
                        Spans::from(Span::raw("Input format:")),
                        Spans::from(Span::raw("  Project: ID Name")),
                        Spans::from(Span::raw("  Task: ID Title")),
                    ];

                    let help = Paragraph::new(help_text)
                        .block(Block::default().borders(Borders::ALL).title("Help"));

                    f.render_widget(help, chunks[1]);
                }
            }

            // Input bar
            let input_text = Text::from(app.input.as_str());
            let input = Paragraph::new(input_text)
                .style(match app.input_mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                })
                .block(Block::default().borders(Borders::ALL).title("Input"));

            f.render_widget(input, chunks[2]);

            // Status message (render over part of the bottom chunk)
            if !app.status_message.is_empty() {
                let status_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .margin(1)
                    .split(chunks[2])[0];

                let status_text = Text::from(app.status_message.as_str());
                let status = Paragraph::new(status_text).style(Style::default().fg(Color::Red));

                f.render_widget(status, status_chunk);
            }

            // Set cursor position when in editing mode
            if let InputMode::Editing = app.input_mode {
                f.set_cursor(chunks[2].x + app.input.len() as u16 + 1, chunks[2].y + 1);
            }
        })?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('a') => {
                            app.input_mode = InputMode::Editing;
                            app.input.clear();
                            app.status_message.clear();
                        }
                        KeyCode::Char('d') => {
                            // Delete the selected item
                            match app.active_tab {
                                AppTab::Projects => {
                                    if let Some(index) = app.projects_state.selected() {
                                        if let Some(project) = app.projects.get(index) {
                                            if let Err(e) = app.storage.delete_project(project.id) {
                                                app.status_message = format!("Error: {}", e);
                                            } else {
                                                app.status_message = "Project deleted.".to_string();
                                                app.projects = app.storage.list_projects()?;
                                                app.projects_state.select(None);
                                                app.tasks.clear();
                                                app.tasks_state.select(None);
                                            }
                                        }
                                    }
                                }
                                AppTab::Tasks => {
                                    if let Some(project_index) = app.projects_state.selected() {
                                        if let Some(task_index) = app.tasks_state.selected() {
                                            if let Some(project) = app.projects.get(project_index) {
                                                if let Some(task) = app.tasks.get(task_index) {
                                                    let task_id = task.id;

                                                    // Load the project, remove the task, and save
                                                    match app.storage.load_project(project.id) {
                                                        Ok(mut loaded_project) => {
                                                            loaded_project.remove_task(task_id);
                                                            app.storage
                                                                .save_project(&loaded_project)?;
                                                            app.status_message =
                                                                "Task deleted.".to_string();

                                                            // Reload tasks
                                                            app.load_project_tasks()?;
                                                        }
                                                        Err(e) => {
                                                            app.status_message =
                                                                format!("Error: {}", e);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Tab => {
                            // Switch tabs
                            app.active_tab = match app.active_tab {
                                AppTab::Projects => AppTab::Tasks,
                                AppTab::Tasks => AppTab::Help,
                                AppTab::Help => AppTab::Projects,
                            };

                            // If switching to Tasks tab, load tasks for the selected project
                            if let AppTab::Tasks = app.active_tab {
                                app.load_project_tasks()?;
                            }
                        }
                        KeyCode::Up => {
                            app.select_previous();
                        }
                        KeyCode::Down => {
                            app.select_next();
                        }
                        KeyCode::Enter => {
                            // Select the current item
                            match app.active_tab {
                                AppTab::Projects => {
                                    if app.projects_state.selected().is_some() {
                                        app.active_tab = AppTab::Tasks;
                                        app.load_project_tasks()?;
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            // Process the input
                            match app.active_tab {
                                AppTab::Projects => {
                                    app.add_project()?;
                                }
                                AppTab::Tasks => {
                                    app.add_task()?;
                                }
                                _ => {}
                            }
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.input.clear();
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
