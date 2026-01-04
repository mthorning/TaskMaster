use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
pub enum Command {
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
pub enum TaskCommand {
  Add {
    /// Create a new task
    #[arg(value_name = "TASK")]
    task: String,
  },
  List(ListArgs),
}

#[derive(Args)]
#[group(id = "list_filter", multiple = false, required = false)]
struct ListArgs {
  /// Include all tasks
  #[arg(long, short = 'a', group = "list_filter", action = clap::ArgAction::SetFalse)]
  all: bool,

  /// Include only completed tasks
  #[arg(long, short = 'c', group = "list_filter", action = clap::ArgAction::SetFalse)]
  completed: bool,
}
