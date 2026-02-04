use crate::tasks::hash_map_task::{HashMapTaskType, Task};
use crate::tasks::{GetTasksFilterOption, TaskList, TaskUpdateAction};
use anyhow::Result;
use console::{Key, StyledObject, Term, style};
use ctrlc;
use log::debug;
use std::{fmt::Write as FmtWrite, io::Write as IoWrite, thread, time::Duration};

#[derive(Clone)]
enum Mode {
  List,
  Edit(String),
  Add(String),
}

pub struct TasksInteract<'a> {
  tasklist: &'a mut TaskList,
  list_option: GetTasksFilterOption,
  term: Term,
  height: usize,
  cursor: usize,
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
      mode: Mode::List,
    }
  }

  pub fn interact(&mut self) -> Result<bool> {
    ctrlc::set_handler(|| {
      Term::stdout().show_cursor().expect("there was an error");
    })?;

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
        Mode::Add(entered_val) => self.add_edit_mode(entered_val, false)?,
        Mode::Edit(entered_val) => self.add_edit_mode(entered_val, true)?,
      }
    }
  }

  fn list_mode(&mut self) -> Result<Option<bool>> {
    self.term.hide_cursor()?;
    let tasks = &self.tasklist.get_tasks(&self.list_option);
    self.render_list(tasks)?;

    let key = self.term.read_key()?;

    debug!("list_mode: {:?}", key);
    match key {
      Key::Char('a') => {
        self.term.clear_last_lines(self.height)?;
        self.height = 0;
        self.mode = Mode::Add(String::new());
      }
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
          .update_task(TaskUpdateAction::Delete, &tasks[self.cursor].description);
      }
      Key::Char('e') => {
        self.term.clear_last_lines(self.height)?;
        self.height = 0;
        self.mode = Mode::Edit(tasks[self.cursor].description.clone());
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
          .update_task(TaskUpdateAction::Toggle, &tasks[self.cursor].description);
      }
      Key::Enter => {
        if !self.tasklist.has_changes() {
          debug!("Enter: tasklist has no change");
          return Ok(Some(false));
        }

        self.render_diff()?;
        if self.confirm("Save changes?")? {
          return Ok(Some(true));
        }
        self.height += 1;
        return Ok(None);
      }
      Key::Escape => {
        if !self.tasklist.has_changes() {
          debug!("Saved: tasklist has no change");
          return Ok(Some(false));
        }

        self.render_diff()?;
        if self.confirm("Discard changes?")? {
          return Ok(Some(false));
        }
        self.height += 1;
        return Ok(None);
      }
      _ => {}
    }
    Ok(None)
  }

  fn add_edit_mode(&mut self, entered_val: String, is_edit: bool) -> Result<()> {
    self.term.show_cursor()?;
    let output = format!("Description: {}", entered_val);
    self.term.write_all(output.as_bytes())?;

    let tasks = &self.tasklist.get_tasks(&self.list_option);
    let key = self.term.read_key()?;

    debug!("add_edit_mode: {:?}", key);
    match key {
      Key::Enter => {
        if self.tasklist.has_task(&entered_val) {
          self.term.clear_line()?;
          self.term.write_all("Task already exists".as_bytes())?;
          thread::sleep(Duration::new(2, 0));
        } else if is_edit {
          let current_desc = &tasks[self.cursor].description;
          self
            .tasklist
            .update_task(TaskUpdateAction::Edit(&entered_val), current_desc);
        } else {
          self.tasklist.add_task(entered_val)?;
        }

        self.mode = Mode::List;
      }
      Key::Escape => {
        if entered_val.is_empty() {
          self.mode = Mode::List;
        } else {
          self.mode = Mode::Edit(String::new());
        }
      }
      Key::Backspace => {
        let mut new_val = entered_val.clone();
        if !new_val.is_empty() {
          new_val.truncate(new_val.len() - 1);
          self.mode = if is_edit {
            Mode::Edit(new_val)
          } else {
            Mode::Add(new_val)
          }
        }
      }
      Key::Char(char) => {
        let val = format!("{}{}", entered_val, char);
        self.mode = if is_edit {
          Mode::Edit(val)
        } else {
          Mode::Add(val)
        }
      }
      _ => {}
    }
    self.term.clear_line()?;

    Ok(())
  }

  fn render_list(&mut self, tasks_to_print: &[Task]) -> Result<()> {
    self.term.clear_last_lines(self.height)?;

    if tasks_to_print.is_empty() {
      self.term.write_all("No tasks here\n".as_bytes())?;
      self.height = 1;
      return Ok(());
    }

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
    self.term.write_all(output.as_bytes())?;

    self.height = output.lines().count();

    Ok(())
  }

  fn render_diff(&mut self) -> Result<()> {
    let mut output = String::new();

    let make_dot = |is_completed: bool| {
      if is_completed {
        style("●".to_string())
      } else {
        style("○".to_string())
      }
    };

    let make_desc = |is_completed: bool, description: String| {
      let mut desc_style = style(description);
      if is_completed {
        desc_style = desc_style.strikethrough();
      }
      desc_style
    };

    for hmt in self
      .tasklist
      .get_hash_map_tasks(&GetTasksFilterOption::AllWithDeleted)
    {
      let task = hmt.get_task();
      match hmt.task_type {
        HashMapTaskType::Existing => {
          let make_coloured =
            |obj: StyledObject<String>, has_changed: bool| -> StyledObject<String> {
              if has_changed { obj.dim() } else { obj.white() }
            };

          let task_dot = make_dot(task.is_completed);
          let task_desc = make_desc(task.is_completed, task.description.to_string());

          let original_task = hmt.get_original_task();

          write!(
            &mut output,
            "  {} ",
            make_coloured(task_dot, task.is_completed == original_task.is_completed)
          )?;

          if task.description != original_task.description {
            write!(
              &mut output,
              "{} ",
              make_desc(task.is_completed, original_task.description.to_string()).dim()
            )?;
          }

          writeln!(
            &mut output,
            "{}",
            make_coloured(task_desc, task.description == original_task.description),
          )?;
        }
        HashMapTaskType::Deleted => {
          let task_str = style(format!(
            "{} {}",
            make_dot(task.is_completed),
            make_desc(task.is_completed, task.description.to_string())
          ))
          .red();

          writeln!(&mut output, "{}", task_str)?;
        }
        HashMapTaskType::Added => {
          let task_str = style(format!(
            "{} {}",
            make_dot(task.is_completed),
            make_desc(task.is_completed, task.description.to_string())
          ))
          .green();

          writeln!(&mut output, "{}", task_str)?;
        }
      }
    }

    self.term.clear_last_lines(self.height)?;
    self.height = output.lines().count();
    self.term.write_all(output.as_bytes())?;

    Ok(())
  }

  fn confirm(&mut self, prompt: &str) -> Result<bool> {
    self.term.write_line(&format!("{} [y/n]", prompt))?;
    if let Key::Char('y') | Key::Enter = self.term.read_key()? {
      return Ok(true);
    }

    Ok(false)
  }
}
