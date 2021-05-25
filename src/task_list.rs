use super::task;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub struct TaskList {
    file : String,
    tasks : Vec<task::Task>,
    prefixes : Vec<String>,
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
        for (pos, task) in self.tasks.iter().enumerate() {
            println!("{:width$} - {}", self.prefixes[pos], task.desc(), width = self.prefix_max_len);
        }
    }

    fn compute_prefixes(&mut self) {
        // Create shortest ids for each task
        self.prefix_max_len = 1;
        self.prefixes = Vec::new();
        for (pos, task) in self.tasks.iter().enumerate() {
            let mut len = 1;
            let mut unique = false;
            let mut prefix = "";

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
            if len > self.prefix_max_len {
                self.prefix_max_len = len
            }
            self.prefixes.push(prefix.to_string());
        }
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
        prefixes : Vec::new(),
        prefix_max_len : 64
    };

    task_list.compute_prefixes();
    task_list
}

