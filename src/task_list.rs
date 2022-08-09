use super::task;
use std::collections::HashMap;
use std::fs::File;
use std::fmt::Write as _;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskListError {
    #[error("Prefix matches more than one task.")]
    AmbiguousPrefix,

    #[error("Prefix matches no tasks.")]
    BadPrefix,

    #[error("Parent prefix matches no tasks.")]
    BadParentPrefix,

    #[error("A task with this id already exits.")]
    DuplicateTask,

    #[error("The task you are trying to remove has children.  Use --force.")]
    RemoveHasChildren,

    // Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub struct TaskList {
    file: String,
    tasks: Vec<task::Task>,
    prefixes: HashMap<String, String>,
    prefix_max_len: usize,
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

impl TaskList {
    pub fn show(&self) {
        println!("Tasks:");
        self.show_tasks(None, "│");
    }

    fn show_tasks(&self, parent_id: Option<&str>, indent: &str) {
        let mut sorted_tasks = Vec::new();
        for task in &self.tasks {
            if task.parent_id().as_deref() == parent_id {
                sorted_tasks.push(task);
            }
        }
        sorted_tasks.sort_by(|a, b| a.timestamp().partial_cmp(&b.timestamp()).unwrap());
        let num_tasks = sorted_tasks.len();
        for (ii, task) in sorted_tasks.iter().enumerate() {
            let last_task = ii == num_tasks - 1;
            if let Some(prefix) = self.prefixes.get(task.id()) {
                // Create list of tags, e.g. [tag1] [tag2] [tag3]
                let mut tags = "".to_string();
                for tag in task.tags() {
                    //tags += &format!("[{}] ", tag);
                    let _ = write!(tags, "[{}] ", tag);
                }

                let indent_item = {
                    let mut a = indent.to_string();
                    a.pop();
                    if last_task {
                        a + "└─ "
                    } else {
                        a + "├─ "
                    }
                };

                println!("{}{}: {}{}", indent_item, prefix, tags, task.desc());

                let next_indent = if last_task {
                    let mut a = indent.to_string();
                    a.pop();
                    a + "   │"
                } else {
                    indent.to_string() + "  │"
                };

                self.show_tasks(Some(task.id()), &next_indent);
            }
        }
    }

    pub fn add_task(
        &mut self,
        parent_id: Option<&str>,
        id: Option<&str>,
        desc: &str,
    ) -> Result<(), TaskListError> {
        // Check if task with this user specified id already exists
        if let Some(id) = id {
            if let Ok(task) = self.get_task(id) {
                if id == task.id() {
                    return Err(TaskListError::DuplicateTask);
                }
            }
        }

        // Check if parent exists
        let full_parent_id;
        match parent_id {
            Some(parent_id) => match self.get_full_id(parent_id) {
                Ok(full_id) => {
                    full_parent_id = Some(full_id);
                }
                Err(_) => {
                    return Err(TaskListError::BadParentPrefix);
                }
            },
            None => {
                full_parent_id = None;
            }
        }

        // Create Task
        let task = task::create(full_parent_id.as_deref(), id, desc);
        let task_id = task.id().to_string();

        // Add Task to Task List
        self.tasks.push(task);
        self.compute_prefixes();

        // Show user added Task information
        let task_prefix: String = match self.prefixes.get(&task_id) {
            Some(prefix) => prefix.to_string(),
            None => task_id.to_string(),
        };
        println!("added task {} ({})", task_prefix, task_id);

        Ok(())
    }

    fn compute_prefixes(&mut self) {
        // Create shortest ids for each task
        // Note: this is a crude, slow implementation (but it works)
        self.prefix_max_len = 1;
        self.prefixes = HashMap::new();
        for (pos, task) in self.tasks.iter().enumerate() {
            let mut len = 1;
            let mut unique = false;
            let mut prefix = "";

            if task.show_full_id() {
                prefix = task.id();
                len = prefix.len();
            } else {
                while !unique && len <= task.id().len() {
                    prefix = &task.id()[..len];
                    unique = true;
                    for (pos2, task2) in self.tasks.iter().enumerate() {
                        if pos == pos2 {
                            continue;
                        }

                        if task2.id().starts_with(prefix) {
                            unique = false;
                            len += 1;
                            break;
                        }
                    }
                }
            }
            if len > self.prefix_max_len {
                self.prefix_max_len = len
            }
            self.prefixes.insert(task.id().to_string(), prefix.to_string());
        }
    }

    pub fn save(&self) {
        let file = File::create(&self.file).expect("cannot open file for write");
        let mut file = BufWriter::new(file);

        let mut sorted_tasks = self.tasks.to_vec();
        sorted_tasks.sort_by(|a, b| a.id().partial_cmp(b.id()).unwrap());

        for task in &sorted_tasks {
            file.write_all((task.to_file_string() + "\n").as_bytes()).expect("cannot write data");
        }
    }

    pub fn remove_task(&mut self, prefix: &str, force: bool) -> Result<(), TaskListError> {
        let full_id = self.get_full_id(prefix)?;

        let children = self.get_children_tasks(&full_id)?;
        let children_ids: Vec<String> = children.into_iter().map(|c| c.id().to_string()).collect();
        if !children_ids.is_empty() {
            if force {
                for id in &children_ids {
                    self.remove_task(id, force)?;
                }
            } else {
                return Err(TaskListError::RemoveHasChildren);
            }
        }

        self.tasks.retain(|task| *task.id() != full_id);
        self.compute_prefixes();

        println!("removed task {} ({})", prefix, full_id);

        Ok(())
    }

    fn get_full_id(&self, prefix: &str) -> Result<String, TaskListError> {
        let mut full_id = None;
        for task in &self.tasks {
            if task.id() == prefix {
                full_id = Some(task.id().to_string());
                break;
            } else if task.id().starts_with(prefix) {
                if full_id.is_some() {
                    // more than one task matches this prefix
                    return Err(TaskListError::AmbiguousPrefix);
                }

                full_id = Some(task.id().to_string());
            }
        }

        match full_id {
            Some(full_id) => Ok(full_id),
            None => Err(TaskListError::BadPrefix),
        }
    }

    pub fn get_task(&mut self, prefix: &str) -> Result<&mut task::Task, TaskListError> {
        let full_id = self.get_full_id(prefix)?;

        for task in &mut self.tasks {
            if task.id().eq(&full_id) {
                return Ok(task);
            }
        }

        Err(TaskListError::BadPrefix)
    }

    pub fn get_children_tasks(
        &mut self,
        prefix: &str,
    ) -> Result<Vec<&mut task::Task>, TaskListError> {
        let mut children = Vec::new();
        let full_id = self.get_full_id(prefix)?;

        for task in &mut self.tasks {
            if task.parent_id().eq(&Some(full_id.to_string())) {
                children.push(task);
            }
        }

        Ok(children)
    }
}

pub fn create_from_file(file: &str) -> TaskList {
    let mut tasks = Vec::new();

    if let Ok(lines) = read_lines(file) {
        for task_string in lines.into_iter().flatten() {
            let task = task::create_from_file_string(&task_string);
            tasks.push(task);
        }
    }

    let mut task_list =
        TaskList { file: file.to_string(), tasks, prefixes: HashMap::new(), prefix_max_len: 64 };

    task_list.compute_prefixes();
    task_list
}
