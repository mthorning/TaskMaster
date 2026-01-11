use crate::tasks::tasklist::*;
use anyhow::Result;
use std::io::{self, Write};

pub struct TaskIO<S: TaskListPersist> {
  storage: S,
}

enum TasksConfirmed {
  None,
  One(usize),
  All,
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

    match self.filter_confirmed(&tasks)? {
      TasksConfirmed::None => {
        println!("No tasks updated");
      }
      TasksConfirmed::One(idx) => {
        if let Some(()) = tasklist.toggle_task(&tasks[idx].description) {
          println!("Task updated");
        }
      }
      TasksConfirmed::All => {
        let mut updated = 0;
        tasks
          .iter()
          .for_each(|task| match tasklist.toggle_task(&task.description) {
            Some(()) => updated += 1,
            None => println!("Unable to update: {}", &task.description),
          });

        println!(
          "{} Task{} updated",
          tasks.len(),
          if tasks.len() == 1 { "" } else { "s" }
        );
      }
    }

    self.storage.save_tasklist(&tasklist)?;

    Ok(())
  }

  fn filter_confirmed(&self, tasks: &Vec<Task>) -> Result<TasksConfirmed> {
    if tasks.len() > 0 {
      println!("Found {} matching tasks:", tasks.len());
    } else {
      println!("Found 0 matching tasks");
      return Ok(TasksConfirmed::None);
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

    if trimmed_answer == "y" {
      return Ok(TasksConfirmed::All);
    }

    if tasks.len() > 1
      && let Ok(selection) = trimmed_answer.parse::<usize>()
    {
      let idx = selection - 1;
      if idx < tasks.len() {
        return Ok(TasksConfirmed::One(idx));
      }
    }

    Ok(TasksConfirmed::None)
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
