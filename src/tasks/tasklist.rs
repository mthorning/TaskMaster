use crate::tasks::hash_map_task::{HashMapTask, HashMapTaskType, Task};
use anyhow::{Result, anyhow};
use log::{debug, info};
use regex::Regex;
use std::{collections::HashMap, mem, sync::Arc};

#[derive(Debug, PartialEq)]
pub struct TaskList {
  tasks: HashMap<Arc<str>, HashMapTask>,
  order_cursor: usize,
}

pub trait TaskListPersist {
  fn load_tasklist(&mut self) -> Result<TaskList>;
  fn save_tasklist(&mut self, tasks: &mut TaskList) -> Result<()>;
}

#[derive(PartialEq)]
pub enum GetTasksFilterOption {
  All,
  AllWithDeleted,
  Completed,
  Incomplete,
}

pub enum TaskUpdateAction<'a> {
  Toggle,
  Delete,
  Edit(&'a str),
}

const MD_RE: &str = r"-\s\[([\sx])\]\s(.+)";

impl TaskList {
  #[cfg(test)]
  fn from(tasks: Vec<Task>) -> TaskList {
    let mut tasklist = TaskList {
      tasks: HashMap::new(),
      order_cursor: tasks.len(),
    };

    tasklist.set_tasks(tasks);

    tasklist
  }

  #[cfg(test)]
  fn set_tasks(&mut self, tasks: Vec<Task>) {
    for (i, task) in tasks.into_iter().enumerate() {
      let hmt = HashMapTask::from(task, i);
      self.tasks.insert(hmt.get_key(), hmt);
    }
  }

  pub fn from_markdown(md_lines: &[String]) -> Result<TaskList> {
    info!("loading tasks from markdown file");
    let mut tasklist = TaskList {
      tasks: HashMap::new(),
      order_cursor: 0,
    };

    for line in md_lines.iter() {
      if let Some((c, d)) = TaskList::get_md_captures(line)? {
        let hmt = HashMapTask::from(
          Task {
            description: d.trim().to_string(),
            is_completed: c != " ",
          },
          tasklist.order_cursor,
        );

        debug!("adding from md: {:?}", hmt);
        tasklist.tasks.insert(hmt.get_key(), hmt);
        tasklist.order_cursor += 1;
      }
    }

    Ok(tasklist)
  }

  pub fn save_to_markdown(&mut self, md_lines: &mut Vec<String>) -> Result<()> {
    let tasks = self.remap_to_original_keys();
    info!("saving tasks to markdown: {:?}", tasks);

    let update_line = |task: Task, line: &mut String| {
      debug!("updating md line for \"{}\"", task.description);
      let check = if task.is_completed { "x" } else { " " };
      *line = format!("- [{}] {}", check, task.description);
    };

    let mut lines_to_remove: Vec<usize> = Vec::new();

    // Update existing tasks
    debug!("updating existing tasks...");
    for (i, line) in md_lines.iter_mut().enumerate() {
      let line_slice: &str = line.as_str();

      if let Some((_, description)) = TaskList::get_md_captures(line_slice)? {
        if let Some(hmt) = tasks.get(description)
          && hmt.task_type != HashMapTaskType::Deleted
        {
          debug!("matched on \"{}\"; writing", &description);
          update_line(hmt.get_task(), line);
        } else {
          lines_to_remove.push(i);
        }
      }
    }

    // Remove deleted tasks
    debug!("removing existing tasks...");
    lines_to_remove.reverse();
    lines_to_remove.into_iter().for_each(|i| {
      debug!("matched on \"{}\"; to be removed", md_lines[i]);
      md_lines.remove(i);
    });

    // Add new tasks
    for hmt in tasks.values() {
      if hmt.task_type == HashMapTaskType::Added {
        debug!("adding line \"{}\"", hmt.get_task().description);
        md_lines.push(String::new());
        let last_line = md_lines.last_mut().unwrap();
        update_line(hmt.get_task(), last_line);
      }
    }

    Ok(())
  }

  pub fn add_task(&mut self, description: String) -> Result<()> {
    let hmt = HashMapTask::new(description, self.order_cursor);

    let key = hmt.get_key();
    if self.tasks.contains_key(&key) {
      return Err(anyhow!("Task already exists"));
    }

    debug!("adding new task {:?}", hmt);
    self.order_cursor += 1;
    self.tasks.insert(key, hmt);

    Ok(())
  }

  pub fn get_hash_map_tasks(&self, list_option: &GetTasksFilterOption) -> Vec<&HashMapTask> {
    let mut hmts = Vec::new();

    for hmt in self.tasks.values() {
      match list_option {
        GetTasksFilterOption::Incomplete => {
          if hmt.task_type != HashMapTaskType::Deleted && !hmt.get_task().is_completed {
            hmts.push(hmt);
          }
        }
        GetTasksFilterOption::Completed => {
          if hmt.task_type != HashMapTaskType::Deleted && hmt.get_task().is_completed {
            hmts.push(hmt);
          }
        }
        GetTasksFilterOption::All => {
          if hmt.task_type != HashMapTaskType::Deleted {
            hmts.push(hmt);
          }
        }
        GetTasksFilterOption::AllWithDeleted => {
          hmts.push(hmt);
        }
      }
    }

    hmts.sort();

    hmts
  }

  pub fn get_tasks(&self, list_option: &GetTasksFilterOption) -> Vec<Task> {
    let hmts = self.get_hash_map_tasks(list_option);
    hmts.into_iter().map(|hmt| hmt.get_task()).collect()
  }

  pub fn update_task(&mut self, action: TaskUpdateAction, description: &str) -> Option<()> {
    if self.tasks.contains_key(description) {
      return match action {
        TaskUpdateAction::Toggle => {
          let hmt = self.tasks.get_mut(description).unwrap();
          hmt.toggle();
          Some(())
        }
        TaskUpdateAction::Delete => match self.tasks.get_mut(description) {
          Some(hmt) => {
            hmt.delete();
            Some(())
          }
          None => None,
        },
        TaskUpdateAction::Edit(new_description) => {
          // using description as a key so need to remove the old and
          // add a new task
          let mut hmt = self.tasks.remove(description).unwrap();
          let new_key = hmt.set_description(new_description);
          self.tasks.insert(new_key, hmt);
          Some(())
        }
      };
    } else {
      println!("Task not found")
    }
    None
  }

  pub fn has_task(&self, description: &str) -> bool {
    self.tasks.contains_key(description)
  }

  pub fn has_changes(&self) -> bool {
    for hmt in self.tasks.values() {
      if hmt.task_type != HashMapTaskType::Existing || hmt.get_task() != hmt.get_original_task() {
        return true;
      }
    }

    false
  }

  fn get_md_captures(haystack: &str) -> Result<Option<(&str, &str)>> {
    let re = Regex::new(MD_RE)?;

    let mut found = None;

    if let Some(caps) = re.captures(haystack)
      && re.is_match(haystack)
      && let (Some(c), Some(d)) = (caps.get(1), caps.get(2))
    {
      found = Some((c.as_str(), d.as_str()))
    }

    Ok(found)
  }

  fn remap_to_original_keys(&mut self) -> HashMap<Arc<str>, HashMapTask> {
    let old_map = mem::take(&mut self.tasks);
    let mut new_map: HashMap<Arc<str>, HashMapTask> = HashMap::with_capacity(old_map.len());
    for (_, hmt) in old_map {
      new_map.insert(hmt.get_original_key(), hmt);
    }

    new_map
  }
}

#[cfg(test)]
#[path = "tasklist_tests.rs"]
mod tasklist_tests;
