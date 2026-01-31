use crate::tasks::{GetTasksFilterOption, Task, TaskList, TaskUpdateAction};
use anyhow::Result;
use console::{Key, Term, style};
use ctrlc;
use std::{fmt::Write as FmtWrite, io::Write as IoWrite, thread, time::Duration};

#[derive(Clone)]
enum Mode {
  List,
  Edit(String),
}

pub struct TasksInteract<'a> {
  tasklist: &'a mut TaskList,
  list_option: GetTasksFilterOption,
  term: Term,
  height: usize,
  cursor: usize,
  has_changes: bool,
  mode: Mode,
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
      mode: Mode::List,
    }
  }

  pub fn interact(&mut self) -> Result<bool> {
    ctrlc::set_handler(|| {
      Term::stdout().show_cursor().expect("there was an error");
    })?;

    self.term.hide_cursor()?;

    let result = match self.render_list_and_read() {
      Ok(ok) => Ok(ok),
      Err(err) => {
        // pretty hacky but I don't want to display
        // this if the user ctrl-c aborts
        if err.to_string() == "read interrupted" {
          return Ok(false);
        }

        Err(err)
      }
    };

    self.term.show_cursor()?;

    result
  }

  fn render_list_and_read(&mut self) -> Result<bool> {
    loop {
      match self.mode.clone() {
        Mode::List => {
          if let Some(should_save) = self.list_mode()? {
            return Ok(should_save);
          }
        }
        Mode::Edit(entered_val) => self.edit_mode(entered_val)?,
      }
    }
  }

  fn list_mode(&mut self) -> Result<Option<bool>> {
    let tasks = &self.tasklist.get_tasks(&self.list_option);
    let output = self.render_list(tasks)?;
    self.term.clear_last_lines(self.height)?;
    self.height = output.lines().count();
    self.term.write_all(output.as_bytes())?;

    let key = self.term.read_key()?;

    match key {
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
        self.mode = Mode::Edit(String::new());
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
        self.has_changes = true;
      }
      Key::Enter => {
        if self.has_changes && self.confirm("Save changes?")? {
          return Ok(Some(true));
        }
        return Ok(Some(false));
      }
      Key::Escape => {
        if self.has_changes && self.confirm("Discard changes?")? {
          return Ok(Some(false));
        }
        return Ok(Some(false));
      }
      _ => {}
    }
    Ok(None)
  }

  fn render_list(&self, tasks_to_print: &[Task]) -> Result<String> {
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

  fn edit_mode(&mut self, entered_val: String) -> Result<()> {
    let output = format!("Description: {}", entered_val);
    self.term.write_all(output.as_bytes())?;

    let tasks = &self.tasklist.get_tasks(&self.list_option);
    let key = self.term.read_key()?;

    match key {
      Key::Enter => {
        let current_desc = &tasks[self.cursor].description;
        if self.tasklist.has_task(&entered_val) {
          self.term.clear_line()?;
          self.term.write_all("Task already exists".as_bytes())?;
          thread::sleep(Duration::new(2, 0));
        } else {
          self
            .tasklist
            .update_task(&TaskUpdateAction::Edit(&entered_val), current_desc);

          self.has_changes = true;
        }

        self.term.clear_line()?;
        self.mode = Mode::List;
      }
      Key::Escape => self.mode = Mode::List,
      Key::Char(char) => {
        self.mode = Mode::Edit(format!("{}{}", entered_val, char));
        self.term.clear_line()?;
      }
      _ => {}
    }

    Ok(())
  }

  fn confirm(&mut self, prompt: &str) -> Result<bool> {
    self.term.write_line(&format!("{} [y/n]", prompt))?;
    if let Key::Char('y') = self.term.read_key()? {
      return Ok(true);
    }

    Ok(false)
  }
}
