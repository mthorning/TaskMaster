use anyhow::{Result, anyhow};
use regex::Regex;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, PartialEq)]
struct HashMapTask {
  is_completed: bool,
  description: Arc<str>,
  order: usize,
}

#[derive(Debug)]
pub struct Task {
  pub is_completed: bool,
  pub description: String,
}

#[derive(Debug, PartialEq)]
pub struct TaskList {
  tasks: HashMap<Arc<str>, HashMapTask>,
  to_be_added: Vec<Arc<str>>,
  to_be_removed: Vec<Arc<str>>,
}

pub trait TaskListPersist {
  fn load_tasklist(&mut self) -> Result<TaskList>;
  fn save_tasklist(&mut self, tasks: &TaskList) -> Result<()>;
}

#[derive(PartialEq)]
pub enum GetTasksFilterOption {
  All,
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
      to_be_added: Vec::new(),
      to_be_removed: Vec::new(),
    };

    tasklist.set_tasks(tasks);

    tasklist
  }

  #[cfg(test)]
  fn set_tasks(&mut self, tasks: Vec<Task>) {
    for (i, task) in tasks.into_iter().enumerate() {
      let description: Arc<str> = Arc::from(task.description);
      self.tasks.insert(
        description.clone(),
        HashMapTask {
          description,
          is_completed: task.is_completed,
          order: i,
        },
      );
    }
  }

  pub fn from_markdown(md_lines: &[String]) -> Result<TaskList> {
    let mut task_list = TaskList {
      tasks: HashMap::new(),
      to_be_added: Vec::new(),
      to_be_removed: Vec::new(),
    };

    let mut cursor = 0;
    for line in md_lines.iter() {
      if let Some((c, d)) = TaskList::get_md_captures(line)? {
        let description = d.trim().to_string();
        let desc_arc: Arc<str> = Arc::from(description);

        task_list.tasks.insert(
          desc_arc.clone(),
          HashMapTask {
            is_completed: c != " ",
            description: desc_arc,
            order: cursor,
          },
        );
        cursor += 1;
      }
    }

    Ok(task_list)
  }

  pub fn save_to_markdown(&self, md_lines: &mut Vec<String>) -> Result<()> {
    let update_line = |desc: &Arc<str>, line: &mut String| {
      if let Some(task) = self.tasks.get(desc) {
        let check = if task.is_completed { "x" } else { " " };
        *line = format!("- [{}] {}", check, task.description);
      }
    };

    let mut lines_to_remove: Vec<usize> = Vec::new();

    // Update existing tasks
    for (i, line) in md_lines.iter_mut().enumerate() {
      let line_slice: &str = line.as_str();

      if let Some((_, description)) = TaskList::get_md_captures(line_slice)? {
        let desc_arc = Arc::from(description);
        if self.tasks.contains_key(&desc_arc) {
          update_line(&desc_arc, line);
        } else {
          lines_to_remove.push(i);
        }
      }
    }

    // Remove deleted tasks
    lines_to_remove.reverse();
    lines_to_remove.into_iter().for_each(|i| {
      md_lines.remove(i);
    });

    // Add new tasks
    for arc_desc in &self.to_be_added {
      md_lines.push(String::new());
      let last_line = md_lines.last_mut().unwrap();
      update_line(arc_desc, last_line);
    }

    Ok(())
  }

  pub fn add_task(&mut self, description: &String) -> Result<()> {
    let desc_arc = Arc::from(description.to_owned());

    if self.tasks.contains_key(&desc_arc) {
      return Err(anyhow!("Task already exists"));
    }

    self.tasks.insert(
      desc_arc.clone(),
      HashMapTask {
        description: desc_arc.clone(),
        is_completed: false,
        order: self.tasks.len(),
      },
    );

    self.to_be_added.push(desc_arc);

    Ok(())
  }

  pub fn get_tasks(&self, list_option: &GetTasksFilterOption) -> Vec<Task> {
    struct HybridTask {
      order: usize,
      task: Task,
    }

    let mut hybrid_tasks: Vec<HybridTask> = Vec::new();

    for h_task in self.tasks.values() {
      hybrid_tasks.push(HybridTask {
        task: Task {
          description: (*h_task.description).to_string(),
          is_completed: h_task.is_completed,
        },
        order: h_task.order,
      });
    }

    hybrid_tasks.sort_by(|a, b| a.order.cmp(&b.order));

    let filtered_hybrid_tasks = match list_option {
      GetTasksFilterOption::All => hybrid_tasks,
      _ => hybrid_tasks
        .into_iter()
        .filter(|hybrid_task| match list_option {
          GetTasksFilterOption::Completed => hybrid_task.task.is_completed,
          GetTasksFilterOption::Incomplete => !hybrid_task.task.is_completed,
          _ => unreachable!(),
        })
        .collect(),
    };

    filtered_hybrid_tasks
      .into_iter()
      .map(|fht| fht.task)
      .collect()
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

  pub fn update_task(&mut self, action: &TaskUpdateAction, description: &str) -> Option<()> {
    if let Some(task) = self.tasks.get_mut(description) {
      return match action {
        TaskUpdateAction::Toggle => {
          task.is_completed = !task.is_completed;
          Some(())
        }
        TaskUpdateAction::Delete => match self.tasks.remove(description) {
          Some(removed_task) => {
            self.to_be_removed.push(removed_task.description);
            Some(())
          }
          None => None,
        },
        TaskUpdateAction::Edit(new_description) => {
          task.description = Arc::from(*new_description);
          Some(())
        }
      };
    } else {
      println!("Task not found")
    }
    None
  }
}

#[cfg(test)]
#[path = "tasklist_tests.rs"]
mod tasklist_tests;
