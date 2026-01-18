use clap::{Args, Subcommand};

#[derive(Args)]
pub struct TaskArgs {
  #[command(subcommand)]
  pub command: TaskCommand,
}

#[derive(Subcommand)]
pub enum TaskCommand {
  /// Add a new task
  #[command(alias = "a")]
  Add { description: String },
  /// List tasks
  #[command(alias = "l")]
  List,
}
