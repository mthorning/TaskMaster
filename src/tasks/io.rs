use crate::tasks::{Task, TaskList};
use anyhow::Result;
use console::{Term, style};
use std::io::{self, Write};

pub fn prompt_user(prompt: &str) -> String {
  println!("{}", prompt);
  if let Err(err) = io::stdout().flush() {
    eprintln!("{}", err);
  }
  let mut answer = String::new();
  if let Err(err) = io::stdin().read_line(&mut answer) {
    eprintln!("{}", err);
  }

  answer.trim().to_owned()
}

pub fn toggle_tasks(tasks: &mut Vec<Task>) -> Result<()> {
  let selection = Vec::new();
  for task in tasks.iter_mut() {
    task.is_completed = selection.contains(&&task.description);
  }
  Ok(())
}

pub fn print_tasks(tasks: &Vec<Task>) {
  let term = Term::stdout();
  let tasks_to_print = TaskList::tasks_to_print(tasks);
  for task in tasks_to_print {
    term
      .write_line(&format!("{}", task))
      .expect("Unable to write line");
  }
}

// pub fn toggle_tasks() {
//   let term = Term::stdout();
//   // unimplemented, this is just nonsense
//   tasks.for_each(|| {
//     let cursor = match selected_idx {
//       Some(j) => {
//         if i == j {
//           style(">").cyan()
//         } else {
//           style(" ")
//         }
//       }
//       None => style(""),
//     };

//     term
//       .write_line(&format!("{} {}", cursor, output))
//       .expect("Unable to write line");
//   });
// }
