use crate::tasks::tasklist::*;
use anyhow::Result;
use std::io::{self, Write};

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

  pub fn list(&mut self, completed: bool, all: bool) -> Result<()> {
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
    let tasks = self.tasklist.get_tasks(list_option);
    if tasks.len() == 0 {
      println!("No{} tasks found", task_type);
    }
    TaskIO::<S>::print_tasks(&tasks, false);

    Ok(())
  }

  pub fn toggle(&mut self, partial_desc: &String, all: bool) -> Result<()> {
    let list_option = if all {
      GetTasksFilterOption::All
    } else {
      GetTasksFilterOption::Incomplete
    };

    let tasks = self
      .tasklist
      .find_by_desc(partial_desc.as_str(), list_option);

    self.make_update(&tasks, &TaskUpdateAction::Toggle)?;

    Ok(())
  }

  pub fn delete(&mut self, partial_desc: &String) -> Result<()> {
    let tasks = self
      .tasklist
      .find_by_desc(partial_desc.as_str(), GetTasksFilterOption::All);

    self.make_update(&tasks, &TaskUpdateAction::Delete)?;

    Ok(())
  }

  pub fn edit(&mut self, partial_desc: &String, new_desc: &String) -> Result<()> {
    let tasks = self
      .tasklist
      .find_by_desc(partial_desc.as_str(), GetTasksFilterOption::All);

    self.make_update(&tasks, &TaskUpdateAction::Edit(new_desc.to_owned()))?;

    Ok(())
  }

  fn make_update(&mut self, tasks: &Vec<Task>, action: &TaskUpdateAction) -> Result<()> {
    match self.confirm_one_or_many(&tasks)? {
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
          tasks.len(),
          if tasks.len() == 1 { "" } else { "s" }
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

  fn confirm_one_or_many(&self, tasks: &Vec<Task>) -> Result<TasksConfirmed> {
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
