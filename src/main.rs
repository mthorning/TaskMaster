use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use env_logger;
use std::fs::{OpenOptions, metadata};
use std::io::{Read, Write};

const TASKS_FILE: &str = "tasks.md";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  Task(TaskArgs),
  // Timer(TimerArgs),
  // Note(NoteArgs),
  Status,
}

#[derive(Args)]
struct TaskArgs {
  #[command(subcommand)]
  command: TaskCommand,
}

#[derive(Subcommand)]
enum TaskCommand {
  Add {
    #[arg(value_name = "TASK")]
    task: String,
  },
  List,
}

fn main() -> Result<()> {
  env_logger::init();
  let cli = Cli::parse();

  match &cli.command {
    Command::Task(task_cmd) => match &task_cmd.command {
      TaskCommand::Add { task } => {
        write_task_to_file(&task)?;
        println!("Task added: {}", task);
      }
      TaskCommand::List => {
        list_tasks()?;
      }
    },
    Command::Status => println!("I don't know yet"),
  }
  Ok(())
}

fn write_task_to_file(task: &str) -> Result<()> {
  let needs_newline = match metadata(TASKS_FILE) {
    Ok(meta) => meta.len() > 0,
    Err(_) => false,
  };

  let mut file = OpenOptions::new()
    .append(true)
    .create(true)
    .open(TASKS_FILE)?;

  let new_line = if needs_newline { "\n" } else { "" };
  let task_str = format!("{}- [ ] {}", new_line, task);

  file.write_all(task_str.as_bytes())?;

  Ok(())
}

fn list_tasks() -> Result<()> {
  let mut file = OpenOptions::new().read(true).open(TASKS_FILE)?;

  let mut contents = String::new();
  file.read_to_string(&mut contents)?;

  println!("{}", contents);

  Ok(())
}
