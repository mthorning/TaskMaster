use anyhow::Result;
use clap::Parser;
use flexi_logger::{FileSpec, Logger, WriteMode};
use log::info;

mod cli;
mod markdown;
mod tasks;

const TASKS_FILE: &str = "tasks.md";

fn main() -> Result<()> {
  let _logger = Logger::try_with_env_or_str("info") // use RUST_LOG=debug for debug level
    .unwrap()
    .log_to_file(FileSpec::default().suppress_timestamp())
    .write_mode(WriteMode::Direct)
    .append()
    .start()
    .unwrap();

  info!("Starting application");

  let cli = cli::Cli::parse();

  let md_file = markdown::File::from(TASKS_FILE);
  let mut task_io = tasks::TaskController::new(md_file)?;

  match &cli.command {
    cli::Command::Tasks(task_cmd) => match &task_cmd.command {
      cli::TaskCommand::Add { description } => task_io.add(description.to_owned())?,
      cli::TaskCommand::List => task_io.list()?,
    },
    cli::Command::Status => unimplemented!(),
  }
  Ok(())
}
