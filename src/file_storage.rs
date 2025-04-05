use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_json;

use crate::error::{Result, TaskMasterError};
use crate::project::Project;
use crate::storage::Storage;
use crate::task::Task;

pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&path)?;
        Ok(FileStorage { base_path: path })
    }

    fn project_path(&self, id: u32) -> PathBuf {
        self.base_path.join(format!("project_{}.json", id))
    }

    fn task_path(&self, project_id: u32, task_id: u32) -> PathBuf {
        self.base_path
            .join(format!("project_{}_task_{}.json", project_id, task_id))
    }
}

impl Storage for FileStorage {
    fn save_project(&mut self, project: &Project) -> Result<()> {
        let path = self.project_path(project.id);
        let json = serde_json::to_string(project)
            .map_err(|e| TaskMasterError::SerializationError(e.to_string()))?;

        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn load_project(&self, id: u32) -> Result<Project> {
        let path = self.project_path(id);
        let mut file = File::open(&path).map_err(|_| TaskMasterError::ProjectNotFound(id))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        serde_json::from_str(&contents)
            .map_err(|e| TaskMasterError::SerializationError(e.to_string()))
    }

    fn list_projects(&self) -> Result<Vec<Project>> {
        let mut projects = Vec::new();

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                if let Some(filename) = path.file_name() {
                    let filename = filename.to_string_lossy();
                    if filename.starts_with("project_") && !filename.contains("task") {
                        // Extract the project ID from the filename
                        if let Ok(id) = filename
                            .strip_prefix("project_")
                            .unwrap_or("")
                            .strip_suffix(".json")
                            .unwrap_or("")
                            .parse::<u32>()
                        {
                            match self.load_project(id) {
                                Ok(project) => projects.push(project),
                                Err(_) => continue, // Skip invalid projects
                            }
                        }
                    }
                }
            }
        }

        Ok(projects)
    }

    fn delete_project(&mut self, id: u32) -> Result<()> {
        let path = self.project_path(id);

        if path.exists() {
            fs::remove_file(path)?;
            Ok(())
        } else {
            Err(TaskMasterError::ProjectNotFound(id))
        }
    }

    fn save_task(&self, project_id: u32, task: &Task) -> Result<()> {
        let path = self.task_path(project_id, task.id);
        let json = serde_json::to_string(task)
            .map_err(|e| TaskMasterError::SerializationError(e.to_string()))?;

        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    fn load_task(&self, project_id: u32, task_id: u32) -> Result<Task> {
        let path = self.task_path(project_id, task_id);
        let mut file = File::open(&path).map_err(|_| TaskMasterError::TaskNotFound(task_id))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        serde_json::from_str(&contents)
            .map_err(|e| TaskMasterError::SerializationError(e.to_string()))
    }

    fn delete_task(&self, project_id: u32, task_id: u32) -> Result<()> {
        let path = self.task_path(project_id, task_id);

        if path.exists() {
            fs::remove_file(path)?;
            Ok(())
        } else {
            Err(TaskMasterError::TaskNotFound(task_id))
        }
    }
}

impl Drop for FileStorage {
    fn drop(&mut self) {
        // Any cleanup logic here
        println!("FileStorage resources cleaned up");
    }
}
