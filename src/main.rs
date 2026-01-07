use anyhow::Result;
use clap::Parser;
use env_logger;

use crate::tasks::{GetTasksFilterOption, TaskListPersist};

mod cli;
mod markdown;
mod tasks;

const TASKS_FILE: &'static str = "tasks.md";

fn main() -> Result<()> {
  env_logger::init();
  let cli = cli::Cli::parse();

  let mut md_file = markdown::File::from(TASKS_FILE);

  match &cli.command {
    cli::Command::Tasks(task_cmd) => match &task_cmd.command {
      cli::TaskCommand::Add { description } => {
        let mut tasklist = md_file.load_tasks()?;
        tasklist.add_task(description.to_owned())?;
        md_file.save_tasks(&tasklist)?;
        println!("Task added");
      }

      cli::TaskCommand::List(args) => {
        let tasklist = md_file.load_tasks()?;

        let mut list_option = GetTasksFilterOption::Incomplete;
        if args.completed {
          list_option = GetTasksFilterOption::Completed;
        } else if args.all {
          list_option = GetTasksFilterOption::All;
        }

        let tasks = tasklist.get_tasks(list_option);

        tasks.iter().enumerate().for_each(|(i, task)| {
          let (status, description) = if task.is_completed {
            ("●", format!("\x1b[9m{}\x1b[0m", task.description))
          } else {
            ("○", task.description.clone())
          }; // can use ◐ for in-progress later

          println!("{}: {} {}", i + 1, status, description)
        });
      }
    },

    cli::Command::Status => println!("I don't know yet"),
  }
  Ok(())
}
