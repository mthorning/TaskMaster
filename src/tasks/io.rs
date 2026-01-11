use crate::tasks::tasklist::*;
use anyhow::Result;
use std::io::{self, Write};

pub struct TaskIO<S: TaskListPersist> {
  storage: S,
}

impl<S: TaskListPersist> TaskIO<S> {
  pub fn new(storage: S) -> TaskIO<S> {
    TaskIO { storage }
  }

  pub fn add(&mut self, task_description: &String) -> Result<()> {
    let mut tasklist = self.storage.load_tasklist()?;
    tasklist.add_task(task_description)?;
    self.storage.save_tasklist(&tasklist)?;
    println!("Task added");

    Ok(())
  }

  pub fn list(&mut self, completed: bool, all: bool) -> Result<()> {
    let tasklist = self.storage.load_tasklist()?;

    let mut list_option = GetTasksFilterOption::Incomplete;
    if completed {
      list_option = GetTasksFilterOption::Completed;
    } else if all {
      list_option = GetTasksFilterOption::All;
    }

    let task_type = match &list_option {
      GetTasksFilterOption::All => "",
      GetTasksFilterOption::Completed => " completed",
      GetTasksFilterOption::Incomplete => " incomplete",
    };
    let tasks = tasklist.get_tasks(list_option);
    if tasks.len() == 0 {
      println!("No{} tasks found", task_type);
    }
    TaskIO::<S>::print_tasks(&tasks, false);

    Ok(())
  }

  pub fn toggle(&mut self, partial_desc: &String, all: bool) -> Result<()> {
    let mut tasklist = self.storage.load_tasklist()?;
    let list_option = if all {
      GetTasksFilterOption::All
    } else {
      GetTasksFilterOption::Incomplete
    };

    let tasks = tasklist.find_by_desc(partial_desc.as_str(), list_option);
    if tasks.len() > 0 {
      println!("Found {} matching tasks:", tasks.len());
    } else {
      println!("Found 0 matching tasks");
      return Ok(());
    }
    TaskIO::<S>::print_tasks(&tasks, tasks.len() > 1);

    if tasks.len() == 1 {
      println!("\nUpdate task? (y/n)");
    } else {
      println!("\nUpdate all tasks? (y/n)\nOr select number of task to update");
    }

    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let trimmed_answer = answer.trim();

    let mut message = String::from("No tasks updated");
    if trimmed_answer == "y" {
      tasks
        .iter()
        .for_each(|task| tasklist.toggle_task(&task.description));

      message = format!(
        "{} Task{} updated",
        tasks.len(),
        if tasks.len() == 1 { "" } else { "s" }
      );
    }

    if tasks.len() > 1
      && let Ok(idx) = trimmed_answer.parse::<usize>()
    {
      if idx > 0 && idx <= tasks.len() {
        tasklist.toggle_task(&tasks[idx - 1].description);
        message = format!("Task updated");
      }
    }

    self.storage.save_tasklist(&tasklist)?;
    println!("{}", message);

    Ok(())
  }

  fn print_tasks(tasks: &Vec<Task>, with_numbers: bool) {
    tasks.iter().enumerate().for_each(|(i, task)| {
      let (status, description) = if task.is_completed {
        ("●", format!("\x1b[9m{}\x1b[0m", task.description))
      } else {
        ("○", task.description.clone())
      }; // can use ◐ for in-progress later

      let numbers = if with_numbers {
        format!("{}: ", i + 1)
      } else {
        String::new()
      };
      println!("{}{} {}", numbers, status, description)
    });
  }
}
