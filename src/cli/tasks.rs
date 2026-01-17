use clap::{Args, Subcommand};

#[derive(Args)]
pub struct TaskArgs {
  #[command(subcommand)]
  pub command: TaskCommand,
}

#[derive(Subcommand)]
pub enum TaskCommand {
  /// Add a new task
  Add { description: String },
  /// List tasks
  #[command(alias = "l")]
  List(ListArgs),
  /// Toggle task(s)
  #[command(alias = "t")]
  Toggle,
  /// Complete a task
  #[command(alias = "c")]
  Complete,
  /// Delete
  #[command(alias = "d")]
  Delete { partial_desc: String },
  /// Edit a task description
  #[command(alias = "e")]
  Edit { partial_desc: String },
}

#[derive(Args)]
#[group(id = "list_filter", multiple = false, required = false)]
pub struct ListArgs {
  /// List all tasks
  #[arg(long, short = 'a', group = "list_filter", action = clap::ArgAction::SetTrue)]
  pub all: bool,

  /// List only completed tasks
  #[arg(long, short = 'c', group = "list_filter", action = clap::ArgAction::SetTrue)]
  pub completed: bool,
}
