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

  let md_file = markdown::File::from(TASKS_FILE);
  let mut task_io = tasks::TaskIO::new(md_file);

  match &cli.command {
    cli::Command::Tasks(task_cmd) => match &task_cmd.command {
      cli::TaskCommand::Add { description } => task_io.add(description)?,
      cli::TaskCommand::List(args) => task_io.list(args.completed, args.all)?,
      cli::TaskCommand::Toggle { partial_desc } => task_io.toggle(&partial_desc, true)?,
      cli::TaskCommand::Complete { partial_desc } => task_io.toggle(partial_desc, false)?,
    },
    cli::Command::Status => unimplemented!(),
  }
  Ok(())
}
