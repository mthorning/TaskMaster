use anyhow::Result;
use std::fs::OpenOptions;
use std::io::{Read, Write};

use crate::tasks::{TaskList, TaskListPersist};

pub struct File<'a> {
  pub path: &'a str,
  lines: Vec<String>,
}

impl<'a> File<'a> {
  pub fn from(path: &'a str) -> File<'a> {
    File {
      path,
      lines: Vec::new(),
    }
  }

  pub fn write_file(&self) -> Result<()> {
    let mut file = OpenOptions::new()
      .write(true)
      .truncate(true)
      .create(true)
      .open(&self.path)?;

    file.write_all(self.lines.join("\n").as_bytes())?;

    Ok(())
  }

  pub fn read_file(&mut self) -> Result<()> {
    let mut file = OpenOptions::new().read(true).open(&self.path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    self.lines = contents
      .split("\n")
      .map(|line| line.trim().to_string())
      .collect();

    Ok(())
  }
}

impl<'a> TaskListPersist for File<'a> {
  fn load_tasklist(&mut self) -> Result<TaskList> {
    self.read_file()?;
    let tasklist = TaskList::from_markdown(&self.lines)?;

    Ok(tasklist)
  }

  fn save_tasklist(&mut self, tasklist: &TaskList) -> Result<()> {
    tasklist.save_to_markdown(&mut self.lines)?;
    self.write_file()
  }
}
