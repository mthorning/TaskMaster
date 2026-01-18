use crate::tasks::{Task, TaskList, TaskUpdateAction};
use anyhow::Result;
use console::{Key, Term, style};
use std::fmt::Write as FmtWrite;
use std::io;
use std::io::Write as IoWrite;

pub fn prompt_user(prompt: &str) -> String {
  let term = Term::stdout();
  println!("{}", prompt);
  if let Err(err) = term.flush() {
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
  for task in tasks {
    term
      .write_line(&format!("{}", task.description))
      .expect("Unable to write line");
  }
}

pub struct TMConsole<'a> {
  tasklist: &'a mut TaskList,
  term: Term,
  height: usize,
  cursor: usize,
}

impl<'a> TMConsole<'a> {
  pub fn new(tasklist: &'a mut TaskList) -> TMConsole<'a> {
    TMConsole {
      tasklist,
      term: Term::stdout(),
      height: 0,
      cursor: 0,
    }
  }

  pub fn tasks_interact(&mut self) -> Result<bool> {
    self.term.hide_cursor()?;
    loop {
      let tasks = &self
        .tasklist
        .get_tasks(super::tasklist::GetTasksFilterOption::All);

      let output = self.render(&tasks)?;
      self.clear()?;
      self.term.write_all(output.as_bytes())?;
      self.term.flush()?;
      self.height = output.lines().count();

      match self.term.read_key()? {
        Key::Char('j') => {
          if self.cursor >= self.height - 1 {
            self.cursor = 0;
          } else {
            self.cursor += 1;
          }
        }
        Key::Char('k') => {
          if self.cursor == 0 {
            self.cursor = self.height - 1;
          } else {
            self.cursor -= 1;
          }
        }
        Key::Char(' ') => {
          let _ = self
            .tasklist
            .update_task(&TaskUpdateAction::Toggle, &tasks[self.cursor].description);
        }
        Key::Enter => return Ok(true),
        Key::Escape => return Ok(false),
        _ => {}
      }
    }
  }

  fn render(&self, tasks_to_print: &Vec<Task>) -> Result<String> {
    let mut output = String::new();

    for (i, task) in tasks_to_print.iter().enumerate() {
      if i == self.cursor {
        write!(&mut output, "{}", style("> ").cyan())?;
      } else {
        write!(&mut output, "  ")?;
      };

      if task.is_completed {
        let description = style(task.description.clone()).strikethrough();
        let task_str = style(format!("● {}", description)).green();
        writeln!(&mut output, "{}", task_str)?;
      } else {
        let task_str = style(format!("○ {}", task.description.clone())).white();
        writeln!(&mut output, "{}", task_str)?;
      }
    }

    Ok(output)
  }

  fn clear(&mut self) -> Result<()> {
    self.term.clear_last_lines(self.height)?;
    self.height = 0;

    Ok(())
  }
}
