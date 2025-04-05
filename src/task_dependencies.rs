use std::collections::{HashMap, HashSet, VecDeque};

use crate::error::{Result, TaskMasterError};
use crate::task::Task;

// Represents a directed graph of task dependencies
#[derive(Default)]
pub struct DependencyGraph {
    // Maps from task ID to the IDs of tasks that depend on it
    dependents: HashMap<u32, HashSet<u32>>,

    // Maps from task ID to the IDs of tasks it depends on
    dependencies: HashMap<u32, HashSet<u32>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        DependencyGraph {
            dependents: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    // Add a dependency relationship: task_id depends on dependency_id
    pub fn add_dependency(&mut self, task_id: u32, dependency_id: u32) -> Result<()> {
        if task_id == dependency_id {
            return Err(TaskMasterError::InvalidOperation(
                "A task cannot depend on itself".to_string(),
            ));
        }

        // Check for circular dependency before adding
        if self.would_create_cycle(task_id, dependency_id) {
            return Err(TaskMasterError::InvalidOperation(
                "Adding this dependency would create a cycle".to_string(),
            ));
        }

        // Add to dependencies map
        self.dependencies
            .entry(task_id)
            .or_insert_with(HashSet::new)
            .insert(dependency_id);

        // Add to dependents map
        self.dependents
            .entry(dependency_id)
            .or_insert_with(HashSet::new)
            .insert(task_id);

        Ok(())
    }

    // Remove a dependency relationship
    pub fn remove_dependency(&mut self, task_id: u32, dependency_id: u32) {
        if let Some(deps) = self.dependencies.get_mut(&task_id) {
            deps.remove(&dependency_id);
        }

        if let Some(deps) = self.dependents.get_mut(&dependency_id) {
            deps.remove(&task_id);
        }
    }

    // Get all tasks that directly depend on the given task
    pub fn get_dependents(&self, task_id: u32) -> HashSet<u32> {
        self.dependents.get(&task_id).cloned().unwrap_or_default()
    }

    // Get all tasks that the given task directly depends on
    pub fn get_dependencies(&self, task_id: u32) -> HashSet<u32> {
        self.dependencies.get(&task_id).cloned().unwrap_or_default()
    }

    // Check if adding a dependency would create a cycle
    fn would_create_cycle(&self, task_id: u32, new_dependency_id: u32) -> bool {
        // If new_dependency_id depends on task_id (directly or indirectly),
        // adding this dependency would create a cycle
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Start from the new dependency
        queue.push_back(new_dependency_id);

        while let Some(current) = queue.pop_front() {
            if current == task_id {
                // Found a path back to task_id, which would create a cycle
                return true;
            }

            if visited.insert(current) {
                // Add all the dependents of current to the queue
                if let Some(deps) = self.dependents.get(&current) {
                    for &dep in deps {
                        queue.push_back(dep);
                    }
                }
            }
        }

        false
    }

    // Get a topological ordering of tasks (if no cycles exist)
    // Fixed to avoid lifetime issues by accepting a reference to tasks and returning task IDs
    pub fn get_execution_order(&self, tasks: &[Task]) -> Result<Vec<u32>> {
        let mut result = Vec::new();
        let mut temp_marks = HashSet::new();
        let mut perm_marks = HashSet::new();

        // Create a set of all task IDs for easy checking
        let task_ids: HashSet<u32> = tasks.iter().map(|t| t.id).collect();

        // Helper function for topological sort (depth-first search)
        fn visit(
            node: u32,
            graph: &DependencyGraph,
            temp_marks: &mut HashSet<u32>,
            perm_marks: &mut HashSet<u32>,
            result: &mut Vec<u32>,
            task_ids: &HashSet<u32>,
        ) -> Result<()> {
            if temp_marks.contains(&node) {
                // Cycle detected
                return Err(TaskMasterError::InvalidOperation(format!(
                    "Circular dependency detected involving task {}",
                    node
                )));
            }

            if !perm_marks.contains(&node) {
                temp_marks.insert(node);

                // Visit all dependencies
                for &dep in &graph.get_dependencies(node) {
                    visit(dep, graph, temp_marks, perm_marks, result, task_ids)?;
                }

                temp_marks.remove(&node);
                perm_marks.insert(node);

                // Add to result if this task is in our input list
                if task_ids.contains(&node) {
                    result.push(node);
                }
            }

            Ok(())
        }

        // Visit each task in the input list
        for task in tasks {
            if !perm_marks.contains(&task.id) {
                visit(
                    task.id,
                    self,
                    &mut temp_marks,
                    &mut perm_marks,
                    &mut result,
                    &task_ids,
                )?;
            }
        }

        Ok(result)
    }

    // Check if all dependencies of a task are met (i.e., all dependencies are complete)
    pub fn are_dependencies_met(&self, task_id: u32, tasks: &[Task]) -> bool {
        let dependencies = self.get_dependencies(task_id);
        if dependencies.is_empty() {
            return true;
        }

        let task_map: HashMap<u32, &Task> = tasks.iter().map(|t| (t.id, t)).collect();

        for &dep_id in &dependencies {
            if let Some(&dep_task) = task_map.get(&dep_id) {
                // Check if dependency is done
                if let crate::task::TaskStatus::Done = dep_task.status {
                    // Dependency is met
                } else {
                    // Dependency is not met
                    return false;
                }
            } else {
                // Dependency task not found, consider it not met
                return false;
            }
        }

        true
    }
}
