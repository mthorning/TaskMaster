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

const MD_RE: &'static str = r"-\s\[([\sx])\]\s(.+)";

impl TaskList {
  #[cfg(test)]
  fn from(tasks: Vec<Task>) -> TaskList {
    let mut tasklist = TaskList {
      tasks: HashMap::new(),
      to_be_added: Vec::new(),
    };

    for (i, task) in tasks.into_iter().enumerate() {
      let description: Arc<str> = Arc::from(task.description);
      tasklist.tasks.insert(
        description.clone(),
        HashMapTask {
          description,
          is_completed: task.is_completed,
          order: i,
        },
      );
    }

    tasklist
  }

  pub fn from_markdown(lines: &Vec<String>) -> Result<TaskList> {
    let mut task_list = TaskList {
      tasks: HashMap::new(),
      to_be_added: Vec::new(),
    };

    let mut cursor = 0;
    for line in lines.iter() {
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

  pub fn to_markdown(&self, lines: &mut Vec<String>) -> Result<()> {
    let update_line = |desc: &str, line: &mut String| {
      if let Some(task) = self.tasks.get(desc) {
        let check = if task.is_completed { "x" } else { " " };
        *line = format!("- [{}] {}", check, task.description);
      }
    };

    for line in lines.iter_mut() {
      let line_slice: &str = line.as_str();

      if let Some((_, description)) = TaskList::get_md_captures(line_slice)? {
        let desc_owned = description.to_owned();
        update_line(&desc_owned, line)
      }
    }

    for arc_desc in &self.to_be_added {
      lines.push(String::new());
      let last_line = lines.last_mut().unwrap();
      update_line(&*arc_desc, last_line);
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

  pub fn get_tasks(&self, list_option: GetTasksFilterOption) -> Vec<Task> {
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

    return filtered_hybrid_tasks
      .into_iter()
      .map(|fht| fht.task)
      .collect();
  }

  fn get_md_captures(haystack: &str) -> Result<Option<(&str, &str)>> {
    let re = Regex::new(MD_RE)?;

    let mut found = None;

    if re.is_match(haystack) {
      if let Some(caps) = re.captures(haystack) {
        match (caps.get(1), caps.get(2)) {
          (Some(c), Some(d)) => found = Some((c.as_str(), d.as_str())),
          _ => {}
        };
      }
    }

    Ok(found)
  }

  pub fn toggle_task(&mut self, description: &str) -> Option<()> {
    if let Some(task) = self.tasks.get_mut(description) {
      task.is_completed = !task.is_completed;
      return Some(());
    } else {
      println!("Task not found")
    }
    None
  }

  pub fn find_by_desc(&self, partial_desc: &str, list_option: GetTasksFilterOption) -> Vec<Task> {
    let tasks = self.get_tasks(list_option);
    tasks
      .into_iter()
      .filter(|task| {
        task
          .description
          .to_lowercase()
          .contains(&partial_desc.to_lowercase())
      })
      .collect()
  }
}

#[cfg(test)]
#[path = "tasklist_tests.rs"]
mod tasklist_tests;
