use clap::{Args, Parser, Subcommand};
use std::fs::{OpenOptions, metadata};
use std::io::{self, Read, Write};

const TASKS_FILE: &str = "tasks.txt";

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
    #[arg(short, long, value_name = "TASK")]
    new: String,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Command::Task(args) => {
            if args.new != "" {
                let result = write_task_to_file(&args.new);
                match result {
                    Ok(_) => println!("Task added: {:?}", args.new),
                    Err(err) => panic!("Error: {:?}", err),
                }
            }
        }
        _ => println!("Not supported"),
    }
}

fn write_task_to_file(task: &str) -> io::Result<()> {
    let needs_newline = match metadata(TASKS_FILE) {
        Ok(meta) => meta.len() > 0,
        Err(_) => false,
    };

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(TASKS_FILE)?;

    if needs_newline {
        file.write_all(b"\n")?;
    }

    file.write_all(task.as_bytes())?;

    Ok(())
}
