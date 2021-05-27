use super::task;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::Path;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskListError {
    #[error("Prefix matches more than one task")]
    AmbiguousPrefix,

    #[error("Prefix matches no tasks")]
    BadPrefix,

    // Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub struct TaskList {
    file : String,
    tasks : Vec<task::Task>,
    prefixes : HashMap<String, String>,
    prefix_max_len : usize
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

impl TaskList {
    pub fn show(&self) {
        println!("Tasks:");
        let mut sorted_tasks = self.tasks.to_vec();
        sorted_tasks.sort_by(|a, b| a.timestamp().partial_cmp(&b.timestamp()).unwrap());
        for task in sorted_tasks {
            match self.prefixes.get(task.id()) {
                Some(prefix) => {
                    println!("{:width$} - {}", prefix, task.desc(), width = self.prefix_max_len);
                }
                None => {}
            }
        }
    }

    pub fn add_task(&mut self, id: Option<&str>, desc: &str) -> Result<(), TaskListError> {
        let task = task::create(id, desc);
        let task_id = task.id().to_string();

        self.tasks.push(task);
        self.compute_prefixes();

        let task_prefix: String;
        match self.prefixes.get(&task_id) {
            Some(prefix) => {
                task_prefix = prefix.to_string();
            }
            None => {
                task_prefix = task_id.to_string();
            }
        }

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

        for task in &self.tasks {
            file.write(task.to_string().as_bytes()).expect("cannot write data");
            file.write("\n".to_string().as_bytes()).expect("cannot write data");
        }
    }

    pub fn remove_task(&mut self, prefix:&str) -> Result<(), TaskListError> {
        let full_id = self.get_full_id(prefix)?;

        self.tasks.retain(|task| *task.id() != full_id);
        self.compute_prefixes();

        println!("removed task {} ({})", prefix, full_id);

        Ok(())
    }

    fn get_full_id(&self, prefix: &str) -> Result<String, TaskListError> {
        let mut full_id = None;
        for task in &self.tasks {
            if task.id().starts_with(prefix) {
                if full_id.is_some() {
                    // more than one task matches this prefix
                    return Err(TaskListError::AmbiguousPrefix);
                }

                full_id = Some(task.id().to_string());
            }
        }

        match full_id {
            Some(full_id) => {
                return Ok(full_id);
            }
            None => {
                return Err(TaskListError::BadPrefix);
            }
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
}

pub fn create_from_file(file: &str) -> TaskList {
    let mut tasks = Vec::new();

    // File hosts must exist in current path before this produces output
    if let Ok(lines) = read_lines(file) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(task_string) = line {
                let task = task::create_from_string(&task_string);
                tasks.push(task);
            }
        }
    }

    let mut task_list = TaskList {
        file: file.to_string(),
        tasks,
        prefixes : HashMap::new(),
        prefix_max_len : 64
    };

    task_list.compute_prefixes();
    task_list
}

