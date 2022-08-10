mod task;
mod task_list;

#[macro_use]
extern crate clap;
use clap::{App, AppSettings, Arg, ArgMatches, Shell, SubCommand};
use std::str::FromStr;

fn get_args() -> clap::App<'static, 'static> {
    App::new("t")
        .settings(&[AppSettings::DisableHelpSubcommand, AppSettings::VersionlessSubcommands])
        .version(crate_version!())
        .author("Trent Lillehaugen <tllilleh@gmail.com>")
        .about("simple todo tracker")
        .subcommand(
            SubCommand::with_name("add")
                .visible_alias("a")
                .about("Create a new task")
                .arg(
                    Arg::with_name("id")
                        .long("id")
                        .value_name("ID")
                        .takes_value(true)
                        .help("Create task with ID, otherwise one will be auto generated"),
                )
                .arg(
                    Arg::with_name("parent_id")
                        .long("parent")
                        .value_name("ID")
                        .takes_value(true)
                        .help("Create task as a sub-task"),
                )
                .arg(
                    Arg::with_name("task")
                        .value_name("DESC")
                        .help("Task description")
                        .multiple(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .visible_alias("r")
                .about("Remove a task")
                .arg(
                    Arg::with_name("id")
                        .value_name("ID")
                        .takes_value(true)
                        .required(true)
                        .help("Task ID to remove"),
                )
                .arg(
                    Arg::with_name("force")
                        .long("force")
                        .required(false)
                        .help("Force remove a task if it has children"),
                ),
        )
        .subcommand(
            SubCommand::with_name("complete")
                .visible_alias("c")
                .about("Complete a task.  That is, check it off.")
                .arg(
                    Arg::with_name("id")
                        .value_name("ID")
                        .takes_value(true)
                        .required(true)
                        .help("Task ID to complete"),
                )
                .arg(
                    Arg::with_name("force")
                        .long("force")
                        .required(false)
                        .help("Force complete a task if it has children"),
                ),
        )
        .subcommand(
            SubCommand::with_name("uncomplete")
                .visible_alias("u")
                .about("Uncomplete a task. That is, uncheck it.")
                .arg(
                    Arg::with_name("id")
                        .value_name("ID")
                        .takes_value(true)
                        .required(true)
                        .help("Task ID to uncomplete"),
                ),
        )
        .subcommand(
            SubCommand::with_name("edit")
                .visible_alias("e")
                .about("Edit a task")
                .arg(
                    Arg::with_name("id")
                        .value_name("ID")
                        .takes_value(true)
                        .required(true)
                        .help("Task ID to edit"),
                )
                .arg(
                    Arg::with_name("task")
                        .value_name("DESC")
                        .help("Task description")
                        .multiple(true)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("tag")
                .visible_alias("t")
                .about("Tag/Untag a task")
                .arg(
                    Arg::with_name("id")
                        .value_name("ID")
                        .takes_value(true)
                        .required(true)
                        .help("Task ID to tag"),
                )
                .arg(
                    Arg::with_name("tags")
                        .value_name("TAG")
                        .help("Tags")
                        .multiple(true)
                        .required(true),
                ),
        )
        .arg(
            Arg::with_name("completions")
                .long("completions")
                .takes_value(true)
                .value_name("SHELL")
                .help(
                    "Generate shell completions for one of: Bash, Fish, Zsh, PowerShell, Elvish.",
                ),
        )
        .arg(
            Arg::with_name("file")
                .long("file")
                .value_name("FILE")
                .takes_value(true)
                .required_unless("completions")
                .help("FILE to use for task list"),
        )
        .arg(
            Arg::with_name("hide-completed")
                .long("hide-completed")
                .help("Don't show completed tasks"),
        )
}

fn main() {
    let matches = get_args().get_matches();

    if let Some(shell) = matches.value_of("completions") {
        match Shell::from_str(shell) {
            Ok(Shell::Bash) => {
                get_args().gen_completions_to("t", Shell::Bash, &mut std::io::stdout())
            }
            Ok(Shell::Fish) => {
                get_args().gen_completions_to("t", Shell::Fish, &mut std::io::stdout())
            }
            Ok(Shell::Zsh) => {
                get_args().gen_completions_to("t", Shell::Zsh, &mut std::io::stdout())
            }
            Ok(Shell::PowerShell) => {
                get_args().gen_completions_to("t", Shell::PowerShell, &mut std::io::stdout())
            }
            Ok(Shell::Elvish) => {
                get_args().gen_completions_to("t", Shell::Elvish, &mut std::io::stdout())
            }
            _ => println!("unknown shell: {}", shell),
        }
        std::process::exit(0);
    }

    let task_file = matches.value_of("file").unwrap();

    match matches.subcommand() {
        ("add", Some(add_matches)) => add_task(task_file, add_matches),
        ("edit", Some(edit_matches)) => edit_task(task_file, edit_matches),
        ("remove", Some(remove_matches)) => remove_task(task_file, remove_matches),
        ("complete", Some(complete_matches)) => complete_task(task_file, complete_matches),
        ("uncomplete", Some(uncomplete_matches)) => uncomplete_task(task_file, uncomplete_matches),
        ("tag", Some(tag_matches)) => tag_task(task_file, tag_matches),
        ("", None) => show_tasks(task_file, &matches),
        _ => unreachable!(),
    }
}

fn show_tasks(task_file: &str, matches: &ArgMatches) {
    // Load Task List
    let tasks = task_list::create_from_file(task_file);
    let hide_completed = matches.is_present("hide-completed");

    // Show Task List
    tasks.show(hide_completed);
}

fn add_task(task_file: &str, matches: &ArgMatches) {
    // Handle Command Line Options
    let parent_id = matches.value_of("parent_id");

    // Concatenate all words into a single description string
    let mut desc = String::from("");
    match matches.values_of("task") {
        None => {}
        Some(words) => {
            for word in words {
                if !desc.is_empty() {
                    desc += " ";
                }
                desc += word;
            }
        }
    }

    // Load Task List
    let mut tasks = task_list::create_from_file(task_file);

    // Add Task
    if let Err(e) = tasks.add_task(parent_id, matches.value_of("id"), &desc) {
        eprintln!("Error: {}", e);
        return;
    }

    // Save Task List
    tasks.save();
}

fn edit_task(task_file: &str, matches: &ArgMatches) {
    // Handle command line options
    // Get ID
    let id = matches.value_of("id").unwrap();

    // Concatenate all words into a single description string
    let mut desc = String::from("");
    match matches.values_of("task") {
        None => {}
        Some(words) => {
            for word in words {
                if !desc.is_empty() {
                    desc += " ";
                }
                desc += word;
            }
        }
    }

    // Load Task List
    let mut tasks = task_list::create_from_file(task_file);

    // Get Task
    let task = match tasks.get_task(id) {
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
        Ok(task) => task,
    };

    // Update description; if none provided on command line open editor with current value as
    // default.
    if desc.is_empty() {
        if let Ok(edited) = edit::edit(&task.desc()) {
            desc = edited;
        }
    }
    // Remove any newlines
    desc = desc.replace('\n', "");

    task.set_desc(&desc);

    // Save Task List
    tasks.save();
}

fn remove_task(task_file: &str, matches: &ArgMatches) {
    // Handle command line options
    let id = matches.value_of("id").unwrap();
    let force = matches.is_present("force");

    // Load Task List
    let mut tasks = task_list::create_from_file(task_file);

    // Remove Task
    if let Err(e) = tasks.remove_task(id, force) {
        eprintln!("Error: {}", e);
        return;
    }

    // Save Task List
    tasks.save();
}

fn complete_task(task_file: &str, matches: &ArgMatches) {
    // Handle command line options
    let id = matches.value_of("id").unwrap();
    let force = matches.is_present("force");

    // Load Task List
    let mut tasks = task_list::create_from_file(task_file);

    // Complete Task
    if let Err(e) = tasks.complete_task(id, force) {
        eprintln!("Error: {}", e);
        return;
    }

    // Save Task List
    tasks.save();
}

fn uncomplete_task(task_file: &str, matches: &ArgMatches) {
    // Handle command line options
    let id = matches.value_of("id").unwrap();

    // Load Task List
    let mut tasks = task_list::create_from_file(task_file);

    // Get Task
    let task = match tasks.get_task(id) {
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
        Ok(task) => task,
    };

    // Unmplete Task
    task.set_complete(false);

    // Save Task List
    tasks.save();
}

fn tag_task(task_file: &str, matches: &ArgMatches) {
    // Handle command line options
    // Get ID
    let id = matches.value_of("id").unwrap();

    // Load Task List
    let mut tasks = task_list::create_from_file(task_file);

    // Get Task
    let task = match tasks.get_task(id) {
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
        Ok(task) => task,
    };

    // Update tags
    match matches.values_of("tags") {
        None => {}
        Some(tags) => {
            for tag in tags {
                if tag.starts_with('-') {
                    // Remove '-' from tag name.
                    let mut chars = tag.chars();
                    chars.next();
                    let tag = chars.as_str();
                    println!("removing tag: {}", tag);
                    task.remove_tag(tag);
                } else {
                    println!("adding tag: {}", tag);
                    task.add_tag(tag);
                }
            }
        }
    }

    // Save Task List
    tasks.save();
}
