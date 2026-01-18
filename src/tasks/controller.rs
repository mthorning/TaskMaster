use crate::tasks::io;
use crate::tasks::tasklist::*;
use anyhow::Result;

pub struct TaskIO<S: TaskListPersist> {
  storage: S,
  tasklist: TaskList,
}

enum TasksConfirmed {
  None,
  One(usize),
  All,
}

impl<S: TaskListPersist> TaskIO<S> {
  pub fn new(mut storage: S) -> Result<TaskIO<S>> {
    let tasklist = storage.load_tasklist()?;
    Ok(TaskIO { storage, tasklist })
  }

  pub fn add(&mut self, task_description: &String) -> Result<()> {
    self.tasklist.add_task(task_description)?;
    self.save()?;
    println!("Task added");

    Ok(())
  }

  pub fn list(&mut self, _completed: bool, _all: bool) -> Result<()> {
    // let mut list_option = GetTasksFilterOption::Incomplete;
    // if completed {
    //   list_option = GetTasksFilterOption::Completed;
    // } else if all {
    //   list_option = GetTasksFilterOption::All;
    // }

    // let task_type = match &list_option {
    //   GetTasksFilterOption::All => "",
    //   GetTasksFilterOption::Completed => " completed",
    //   GetTasksFilterOption::Incomplete => " incomplete",
    // };
    // let tasks = self.tasklist.get_tasks(list_option);
    // if tasks.len() == 0 {
    //   println!("No{} tasks found", task_type);
    // }
    // io::print_tasks(&tasks);
    let mut console = io::TMConsole::new(&mut self.tasklist);
    let should_save = console.tasks_interact()?;
    if should_save {
      self.save()?;
    }

    Ok(())
  }

  pub fn toggle(&mut self, all: bool) -> Result<()> {
    let list_option = if all {
      GetTasksFilterOption::All
    } else {
      GetTasksFilterOption::Incomplete
    };

    // let tasks = self
    //   .tasklist
    //   .find_by_desc(partial_desc.as_str(), list_option);
    //
    // self
    //   .tasklist
    //   .set_tasks(io::toggle_tasks(self.tasklist.get_tasks(list_option)));

    let mut tasks = self.tasklist.get_tasks(list_option);
    io::toggle_tasks(&mut tasks)?;
    self.tasklist.set_tasks(tasks);

    // self.make_update(&tasks, &TaskUpdateAction::Toggle, true)?;

    Ok(())
  }

  pub fn delete(&mut self, partial_desc: &String) -> Result<()> {
    let tasks = self
      .tasklist
      .find_by_desc(partial_desc.as_str(), GetTasksFilterOption::All);

    self.make_update(&tasks, &TaskUpdateAction::Delete, true)?;

    Ok(())
  }

  pub fn edit(&mut self, partial_desc: &String) -> Result<()> {
    let tasks = self
      .tasklist
      .find_by_desc(partial_desc.as_str(), GetTasksFilterOption::All);

    self.make_update(&tasks, &TaskUpdateAction::Edit, false)?;

    Ok(())
  }

  fn make_update(
    &mut self,
    tasks: &Vec<Task>,
    action: &TaskUpdateAction,
    allow_multiple: bool,
  ) -> Result<()> {
    match self.confirm_one_or_many(&tasks, allow_multiple, action)? {
      TasksConfirmed::None => {
        println!("No tasks updated");
      }
      TasksConfirmed::One(idx) => {
        if let Some(()) = self.tasklist.update_task(action, &tasks[idx].description) {
          println!("Task updated");
        }
      }
      TasksConfirmed::All => {
        let mut updated = 0;
        tasks.iter().for_each(
          |task| match self.tasklist.update_task(action, &task.description) {
            Some(()) => updated += 1,
            None => println!("Unable to update: {}", &task.description),
          },
        );

        println!(
          "{} Task{} updated",
          updated,
          if updated == 1 { "" } else { "s" }
        );
      }
    }

    self.save()?;

    Ok(())
  }

  fn save(&mut self) -> Result<()> {
    self.storage.save_tasklist(&self.tasklist)?;

    Ok(())
  }

  fn confirm_one_or_many(
    &self,
    tasks: &Vec<Task>,
    allow_multiple: bool,
    action: &TaskUpdateAction,
  ) -> Result<TasksConfirmed> {
    let action_str = match action {
      TaskUpdateAction::Toggle => "update",
      TaskUpdateAction::Edit => "edit",
      TaskUpdateAction::Delete => "delete",
    };

    let tasks_len = tasks.len();
    match tasks_len {
      0 => {
        println!("Found 0 matching tasks");
        return Ok(TasksConfirmed::None);
      }
      1 => {
        println!("Found 1 matching task:");
        io::print_tasks(&tasks);
        let answer = io::prompt_user(&format!("\n{} task? (y/n)", action_str));

        if answer == "y" {
          return Ok(TasksConfirmed::One(0));
        }
      }
      _ => {
        let selected_num: String;
        if allow_multiple {
          println!("Found {} matching tasks:", tasks_len);
          io::print_tasks(&tasks);
          let answer = io::prompt_user(&format!(
            "\n{} all tasks? (y/n)\nOr select number of task to update",
            action_str
          ));

          if answer == "y" {
            return Ok(TasksConfirmed::All);
          }
          selected_num = answer;
        } else {
          println!("Found {} matching tasks:", tasks_len);
          io::print_tasks(&tasks);
          selected_num = io::prompt_user(&format!("\nSelect number of task to {}", action_str));
        }

        if let Ok(selection) = selected_num.parse::<usize>() {
          let idx = selection - 1;
          if idx < tasks_len {
            return Ok(TasksConfirmed::One(idx));
          }
        }
      }
    }
    Ok(TasksConfirmed::None)
  }
}
