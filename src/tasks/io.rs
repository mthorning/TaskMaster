use crate::tasks::tasklist::*;
use anyhow::Result;

pub fn add<T: TaskListPersist>(storage: &mut T, task_description: &String) -> Result<()> {
  let mut tasklist = storage.load_tasks()?;
  tasklist.add_task(task_description)?;
  storage.save_tasks(&tasklist)?;
  println!("Task added");

  Ok(())
}

pub fn list<T: TaskListPersist>(storage: &mut T, completed: bool, all: bool) -> Result<()> {
  let tasklist = storage.load_tasks()?;

  let mut list_option = GetTasksFilterOption::Incomplete;
  if completed {
    list_option = GetTasksFilterOption::Completed;
  } else if all {
    list_option = GetTasksFilterOption::All;
  }

  let tasks = tasklist.get_tasks(list_option);

  tasks.iter().enumerate().for_each(|(i, task)| {
    let (status, description) = if task.is_completed {
      ("●", format!("\x1b[9m{}\x1b[0m", task.description))
    } else {
      ("○", task.description.clone())
    }; // can use ◐ for in-progress later

    println!("{}: {} {}", i + 1, status, description)
  });

  Ok(())
}
