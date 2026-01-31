use anyhow::Result;
use clap::Parser;

mod cli;
mod markdown;
mod tasks;

const TASKS_FILE: &str = "tasks.md";

fn main() -> Result<()> {
  env_logger::init();
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
