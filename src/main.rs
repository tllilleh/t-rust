mod task;
mod task_list;

#[macro_use]
extern crate clap;
use clap::{Arg, App, SubCommand, ArgMatches};

fn main() {
    let matches = App::new("t")
        .version(crate_version!())
        .author("Trent Lillehaugen <tllilleh@gmail.com>")
        .about("simple todo tracker")
        .subcommand(SubCommand::with_name("add")
            .alias("a")
            .about("Create a new task")
            .arg(Arg::with_name("id")
                .long("id")
                .value_name("ID")
                .takes_value(true)
                .help("Create task with ID, otherwise one will be auto generated"))
            .arg(Arg::with_name("parent_id")
                .long("parent")
                .value_name("ID")
                .takes_value(true)
                .help("Create task as a sub-task"))
            .arg(Arg::with_name("task")
                .value_name("DESC")
                .help("Task description")
                .multiple(true)
                .required(true)))
        .subcommand(SubCommand::with_name("remove")
            .alias("r")
            .about("Remove a task")
            .arg(Arg::with_name("id")
                .value_name("ID")
                .takes_value(true)
                .required(true)
                .help("Task ID to remove"))
            .arg(Arg::with_name("force")
                .long("force")
                .required(false)
                .help("Force remove a task if it has children")))
        .subcommand(SubCommand::with_name("edit")
            .alias("e")
            .about("Edit a task")
            .arg(Arg::with_name("id")
                .value_name("ID")
                .takes_value(true)
                .required(true)
                .help("Task ID to edit"))
            .arg(Arg::with_name("task")
                .value_name("DESC")
                .help("Task description")
                .multiple(true)
                .required(true)))
        .subcommand(SubCommand::with_name("test"))
        .get_matches();

    match matches.subcommand() {
        ("add", Some(add_matches)) => add_task(&add_matches),
        ("edit", Some(edit_matches)) => edit_task(&edit_matches),
        ("remove", Some(remove_matches)) => remove_task(&remove_matches),
        ("test", Some(test_matches)) => test(&test_matches),
        ("", None) => show_tasks(),
        _ => unreachable!(),
    }
}

fn show_tasks() {
    let tasks = task_list::create_from_file("todo.txt");
    tasks.show();
}

fn add_task(matches: &ArgMatches)
{
    // Handle Command Line Options
    let parent_id = matches.value_of("parent_id");

    // Concatenate all words into a single description string
    let mut desc = String::from("");
    match matches.values_of("task") {
        None => {},
        Some(words) => {
            for word in words {
                if desc.len() > 0 {
                    desc += " ";
                }
                desc += word;
            }
        }
    }

    // Load Task List
    let mut tasks = task_list::create_from_file("todo.txt");

    // Add Task
    match tasks.add_task(parent_id, matches.value_of("id"), &desc)
    {
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
        Ok(_) => {}
    }

    // Save Task List
    tasks.save();
}

fn edit_task(matches: &ArgMatches)
{
    // Handle command line options
    // Get ID
    let id = matches.value_of("id").unwrap();

    // Concatenate all words into a single description string
    let mut desc = String::from("");
    match matches.values_of("task") {
        None => {},
        Some(words) => {
            for word in words {
                if desc.len() > 0 {
                    desc += " ";
                }
                desc += word;
            }
        }
    }

    // Load Task List
    let mut tasks = task_list::create_from_file("todo.txt");

    // Get Task
    let task = match tasks.get_task(id) {
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
        Ok(task) => {task}
    };

    // Update description
    task.set_desc(&desc);

    // Save Task List
    tasks.save();
}

fn remove_task(matches: &ArgMatches)
{
    // Handle command line options
    let id = matches.value_of("id").unwrap();
    let force = matches.is_present("force");

    // Load Task List
    let mut tasks = task_list::create_from_file("todo.txt");

    // Remove Task
    match tasks.remove_task(id, force) {
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
        Ok(_) => { }
    }

    // Save Task List
    tasks.save();
}

fn test(_matches: &ArgMatches) {
    let mut tasks = Vec::new();

    tasks.push(task::create(None, None, "one"));
    tasks.push(task::create(None, None, "two"));
    tasks.push(task::create(None, None, "three"));

    let json = r#"
        {
            "id": "test"
        }
    "#;

    let imported_task : task::Task = serde_json::from_str(json).unwrap();
    tasks.push(imported_task);

    // Serialize it to a JSON string.
    let mut jsons = Vec::new();
    for task in &tasks {
        jsons.push(task.to_string());
    }

    let mut tasks2 = Vec::new();

    for json in &jsons {
        println!("{}", json);
        tasks2.push(task::create_from_string(json));
    }

    for task in &tasks2 {
        println!("{}", task.to_string());
    }
}

