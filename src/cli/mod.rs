use clap::{Parser, Subcommand};

pub mod tasks;
pub use tasks::*;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  #[command(alias = "t")]
  Tasks(tasks::TaskArgs),
  // Timer(TimerArgs),
  // Note(NoteArgs),
  Status,
}
