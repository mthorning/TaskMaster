use anyhow::Result;
use clap::Parser;
use env_logger;

use crate::tasklist::{GetTasksFilterOption, TaskList};

mod cli;
mod markdown;
mod tasklist;

const TASKS_FILE: &'static str = "tasks.md";

fn main() -> Result<()> {
  env_logger::init();
  let cli = cli::Cli::parse();

  let md_file = markdown::File::from(TASKS_FILE);

  match &cli.command {
    cli::Command::Tasks(task_cmd) => match &task_cmd.command {
      cli::TaskCommand::Add { task } => {
        let task_str = format!("- [ ] {}", task);
        md_file.append_to_file(&task_str)?;
        println!("Added task: {}", task);
      }
      cli::TaskCommand::List(args) => {
        let contents = md_file.get_contents()?;

        let tasklist = TaskList::from_string(&contents);

        let mut list_option = GetTasksFilterOption::Incomplete;
        if args.completed {
          list_option = GetTasksFilterOption::Completed;
        } else if args.all {
          list_option = GetTasksFilterOption::All;
        }

        let tasks = tasklist.get_tasks(list_option);
        tasks.iter().enumerate().for_each(|(i, task)| {
          let check = if task.is_completed { "✓" } else { " " };
          println!("{}: {} {}", i + 1, check, task.description)
        });
      }
    },
    cli::Command::Status => println!("I don't know yet"),
  }
  Ok(())
}
