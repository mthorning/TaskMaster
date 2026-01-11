use anyhow::Result;
use clap::Parser;
use env_logger;

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
      cli::TaskCommand::Add { description } => tasks::add(&mut md_file, description)?,
      cli::TaskCommand::List(args) => tasks::list(&mut md_file, args.completed, args.all)?,
    },
    cli::Command::Status => unimplemented!(),
  }
  Ok(())
}
