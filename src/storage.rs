use crate::error::Result;
use crate::project::Project;
use crate::task::Task;

pub trait Storage {
    fn save_project(&mut self, project: &Project) -> Result<()>;
    fn load_project(&self, id: u32) -> Result<Project>;
    fn list_projects(&self) -> Result<Vec<Project>>;
    fn delete_project(&mut self, id: u32) -> Result<()>;

    // Task methods
    fn save_task(&self, project_id: u32, task: &Task) -> Result<()>;
    fn load_task(&self, project_id: u32, task_id: u32) -> Result<Task>;
    fn delete_task(&self, project_id: u32, task_id: u32) -> Result<()>;
}
