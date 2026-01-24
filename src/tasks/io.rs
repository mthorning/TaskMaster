use crate::tasks::{GetTasksFilterOption, Task, TaskList, TaskUpdateAction};
use anyhow::Result;
use console::{Key, Term, style};
use ctrlc;
use std::{fmt::Write as FmtWrite, io::Write as IoWrite};

enum Mode {
  Normal,
  Insert,
}

enum OnInsert {
  Edit,
  Add,
}

pub struct TasksInteract<'a> {
  tasklist: &'a mut TaskList,
  list_option: GetTasksFilterOption,
  term: Term,
  height: usize,
  cursor: usize,
  has_changes: bool,
  mode: Mode,
  on_insert: OnInsert,
  insert_prompt: String,
  insert_value: String,
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
      mode: Mode::Normal,
      on_insert: OnInsert::Add,
      insert_prompt: String::new(),
      insert_value: String::new(),
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
      let tasks = &self.tasklist.get_tasks(&self.list_option);
      match self.mode {
        Mode::Normal => {
          if let Some(should_save) = self.normal_mode(tasks)? {
            return Ok(should_save);
          }
        }
        Mode::Insert => self.insert_mode(tasks)?,
      }
    }
  }

  fn normal_mode(&mut self, tasks: &[Task]) -> Result<Option<bool>> {
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
        self.on_insert = OnInsert::Edit;
        self.term.clear_last_lines(self.height)?;
        self.insert_prompt = "Description: ".to_string();
        self.mode = Mode::Insert;
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

  fn insert_mode(&mut self, tasks: &[Task]) -> Result<()> {
    self.term.clear_line()?;
    let output = format!("{}{}", self.insert_prompt, self.insert_value);
    self.term.write_all(output.as_bytes())?;

    let key = self.term.read_key()?;

    match key {
      Key::Enter => {
        match self.on_insert {
          OnInsert::Add => unimplemented!(),
          OnInsert::Edit => {
            let current_desc = &tasks[self.cursor].description;
            if current_desc != &self.insert_value {
              self
                .tasklist
                .update_task(&TaskUpdateAction::Edit(&self.insert_value), current_desc);

              self.has_changes = true;
            }
          }
        }

        self.term.clear_line()?;
        self.mode = Mode::Normal;
      }
      Key::Escape => unimplemented!(),
      Key::Char(char) => {
        self.insert_value = format!("{}{}", self.insert_value, char);
      }
      _ => {}
    }

    Ok(())
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

  fn confirm(&mut self, prompt: &str) -> Result<bool> {
    self.term.write_line(&format!("{} [y/n]", prompt))?;
    if let Key::Char('y') = self.term.read_key()? {
      return Ok(true);
    }

    Ok(false)
  }
}
