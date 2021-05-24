mod task;
#[macro_use]
extern crate clap;
use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("t")
        .version(crate_version!())
        .author("Trent Lillehaugen <tllilleh@gmail.com>")
        .about("simple todo tracker")
        .subcommand(SubCommand::with_name("add")
            .about("Create a new task")
            .arg(Arg::with_name("id")
                .long("id")
                .value_name("ID")
                .takes_value(true)
                .help("Create task with given ID"))
            .arg(Arg::with_name("task")
                .value_name("DESC")
                .help("Task description")
                .multiple(true)
                .required(true)))
        .subcommand(SubCommand::with_name("test"))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("add") {
        let id = matches.value_of("id");
        match id {
            None => {},
            Some(s) => {
                println!("Your task id is {}.", s)
            }
        }

        let id = matches.values_of("task");
        match id {
            None => {},
            Some(s) => {
                println!("Your task description is {:?}.", s)
            }
        }

        add_task();
    } else if let Some(matches) = matches.subcommand_matches("test") {
        test();
    } else {
        show_tasks();
    }
}

fn show_tasks() {
    println!("TODO: show tasks");
}

fn add_task()
{
    println!("TODO: add task");
}

fn test() {
    let mut tasks = Vec::new();

    tasks.push(task::create("one"));
    tasks.push(task::create("two"));
    tasks.push(task::create("three"));

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

