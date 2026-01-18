use crate::tasks::{GetTasksFilterOption, Task, TaskList, TaskUpdateAction};
use anyhow::Result;
use console::{Key, Term, style};
use ctrlc;
use std::{fmt::Write as FmtWrite, io::Write as IoWrite};

pub struct TasksInteract<'a> {
  tasklist: &'a mut TaskList,
  list_option: GetTasksFilterOption,
  term: Term,
  height: usize,
  cursor: usize,
  has_changes: bool,
}

impl<'a> TasksInteract<'a> {
  pub fn new(tasklist: &'a mut TaskList) -> TasksInteract<'a> {
    TasksInteract {
      tasklist,
      list_option: GetTasksFilterOption::All,
      term: Term::stdout(),
      height: 0,
      cursor: 0,
      has_changes: false,
    }
  }

  pub fn interact(&mut self) -> Result<bool> {
    match self.run_loop() {
      Ok(ok) => Ok(ok),
      Err(err) => {
        // pretty hacky but I don't want to display
        // this if the user ctrl-c aborts
        if err.to_string() == "read interrupted" {
          return Ok(false);
        }
        Err(err)
      }
    }
  }

  fn run_loop(&mut self) -> Result<bool> {
    self.term.hide_cursor()?;

    ctrlc::set_handler(|| {
      Term::stdout().show_cursor().expect("there was an error");
    })?;

    loop {
      let tasks = &self.tasklist.get_tasks(&self.list_option);

      let output = self.render(tasks)?;
      self.term.clear_last_lines(self.height)?;
      self.height = output.lines().count();
      self.term.write_all(output.as_bytes())?;
      self.term.flush()?;

      match self.term.read_key()? {
        Key::Char('c') => {
          if let GetTasksFilterOption::Completed = self.list_option {
            self.list_option = GetTasksFilterOption::All;
          } else {
            self.list_option = GetTasksFilterOption::Completed;
          }
        }
        Key::Char('d') => {
          self
            .tasklist
            .update_task(&TaskUpdateAction::Delete, &tasks[self.cursor].description);
        }
        Key::Char('e') => {
          self.term.clear_last_lines(self.height)?;
          self.height = 0;
          let new_desc = self.prompt("Description: ")?;
          self.term.clear_last_lines(1)?;
          let current_desc = &tasks[self.cursor].description;
          if current_desc != &new_desc {
            self
              .tasklist
              .update_task(&TaskUpdateAction::Edit(&new_desc), current_desc);

            self.has_changes = true;
          }
        }
        Key::Char('i') => {
          if let GetTasksFilterOption::Incomplete = self.list_option {
            self.list_option = GetTasksFilterOption::All;
          } else {
            self.list_option = GetTasksFilterOption::Incomplete;
          }
        }
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
          self
            .tasklist
            .update_task(&TaskUpdateAction::Toggle, &tasks[self.cursor].description);
        }
        Key::Enter => {
          self.term.clear_last_lines(self.height)?;
          self.height = 0;
          self.term.show_cursor()?;
          if self.has_changes {
            if self.confirm("Save changes?")? {
              return Ok(true);
            } else {
              self.term.clear_last_lines(1)?;
              continue;
            }
          }
          return Ok(false);
        }
        Key::Escape => {
          self.term.clear_last_lines(self.height)?;
          self.height = 0;
          self.term.show_cursor()?;
          if self.has_changes {
            if self.confirm("Discard changes?")? {
              return Ok(false);
            } else {
              self.term.clear_last_lines(1)?;
              continue;
            }
          }
          return Ok(false);
        }
        _ => {}
      }
    }
  }

  fn render(&self, tasks_to_print: &[Task]) -> Result<String> {
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

  fn prompt(&mut self, prompt: &str) -> Result<String> {
    self.term.write_all(prompt.as_bytes())?;
    self.term.flush()?;
    let response = self.term.read_line()?;

    Ok(response.trim().to_owned())
  }

  fn confirm(&mut self, prompt: &str) -> Result<bool> {
    self.term.write_line(&format!("{} [y/n]", prompt))?;
    self.term.flush()?;
    if let Key::Char('y') = self.term.read_key()? {
      return Ok(true);
    }

    Ok(false)
  }
}
