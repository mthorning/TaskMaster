use crate::tasks::io;
use crate::tasks::tasklist::*;
use anyhow::Result;

pub struct TaskController<S: TaskListPersist> {
  storage: S,
  tasklist: TaskList,
}

impl<S: TaskListPersist> TaskController<S> {
  pub fn new(mut storage: S) -> Result<TaskController<S>> {
    let tasklist = storage.load_tasklist()?;
    Ok(TaskController { storage, tasklist })
  }

  pub fn add(&mut self, task_description: String) -> Result<()> {
    self.tasklist.add_task(task_description)?;
    self.save()?;
    println!("Task added");

    Ok(())
  }

  pub fn list(&mut self) -> Result<()> {
    let mut console = io::TasksInteract::new(&mut self.tasklist);
    let should_save = io::TasksInteract::interact(&mut console)?;
    if should_save {
      self.save()?;
    }

    Ok(())
  }

  fn save(&mut self) -> Result<()> {
    self.storage.save_tasklist(&mut self.tasklist)?;

    Ok(())
  }
}
