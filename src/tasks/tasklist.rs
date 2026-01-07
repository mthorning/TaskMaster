use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Task {
  pub is_completed: bool,
  pub description: String,
  order: usize,
}

#[derive(Debug, PartialEq)]
pub struct TaskList {
  pub tasks: HashMap<String, Task>,
  cursor: usize,
}

pub trait TaskListPersist {
  fn load_tasks(&mut self) -> Result<TaskList>;
  fn save_tasks(&self, tasks: &TaskList) -> Result<()>;
}

#[derive(PartialEq)]
pub enum GetTasksFilterOption {
  All,
  Completed,
  Incomplete,
}

const MD_RE: &'static str = r"-\s\[([\sxX])\]\s(.+)";

impl TaskList {
  pub fn from_markdown(lines: &Vec<String>) -> Result<TaskList> {
    let mut task_list = TaskList {
      tasks: HashMap::new(),
      cursor: 0,
    };

    for line in lines.iter() {
      if let Some((c, d)) = get_md_captures(line)? {
        let description = d.trim().to_string();
        task_list.tasks.insert(
          description.clone(),
          Task {
            is_completed: c != " ",
            description,
            order: task_list.cursor,
          },
        );
        task_list.cursor += 1;
      }
    }

    Ok(task_list)
  }

  pub fn to_markdown(lines: &mut Vec<String>) -> Result<()> {
    for line in lines {}

    Ok(())
  }

  pub fn add_task(&mut self, description: String) -> Result<()> {
    if self.tasks.contains_key(&description) {
      return Err(anyhow!("Task already exists"));
    }

    self.tasks.insert(
      description.clone(),
      Task {
        description,
        is_completed: false,
        order: self.cursor,
      },
    );

    self.cursor += 1;

    Ok(())
  }

  pub fn get_tasks(&self, list_option: GetTasksFilterOption) -> Vec<Task> {
    let mut cloned_tasks: Vec<Task> = match list_option {
      GetTasksFilterOption::All => self.tasks.values().cloned().collect(),
      _ => self
        .tasks
        .values()
        .filter(|task| match list_option {
          GetTasksFilterOption::Completed => task.is_completed,
          GetTasksFilterOption::Incomplete => !task.is_completed,
          _ => unreachable!(),
        })
        .cloned()
        .collect(),
    };

    cloned_tasks.sort_by(|a, b| a.order.cmp(&b.order));
    return cloned_tasks;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_from_markdown() {
    let test_lines = vec![
      String::from("hello"),
      String::from("- this is a note"),
      String::from("- [ ] incomplete task"),
      String::from("- [x] complete task  "),
      String::from("nothing"),
    ];

    let result = TaskList::from_markdown(&test_lines);
    let expected = TaskList {
      cursor: 2,
      tasks: HashMap::from([
        (
          "incomplete task".to_string(),
          Task {
            is_completed: false,
            description: "incomplete task".to_string(),
            order: 0,
          },
        ),
        (
          "complete task".to_string(),
          Task {
            is_completed: true,
            description: "complete task".to_string(),
            order: 1,
          },
        ),
      ]),
    };

    assert_eq!(expected, result.unwrap());
  }

  #[test]
  fn test_add_task() {
    let mut tasklist = TaskList {
      cursor: 0,
      tasks: HashMap::new(),
    };

    if let Err(err) = tasklist.add_task("test description".to_string()) {
      panic!("{}", err);
    }

    let task = tasklist
      .tasks
      .get("test description")
      .unwrap_or_else(|| panic!("Task is None"));

    assert_eq!(task.description, "test description");
    assert!(task.is_completed == false);
    assert_eq!(task.order, 0);
    assert_eq!(tasklist.cursor, 1);
  }

  #[test]
  fn test_list_tasks() {
    let one = "one".to_string();
    let two = "two".to_string();
    let three = "three".to_string();

    let tasklist = TaskList {
      cursor: 0,
      tasks: HashMap::from([
        (
          one.clone(),
          Task {
            is_completed: false,
            description: one,
            order: 0,
          },
        ),
        (
          two.clone(),
          Task {
            is_completed: true,
            description: two,
            order: 1,
          },
        ),
        (
          three.clone(),
          Task {
            is_completed: false,
            description: three,
            order: 2,
          },
        ),
      ]),
    };

    let all_tasks = tasklist.get_tasks(GetTasksFilterOption::All);
    assert_eq!(tasklist.tasks.len(), all_tasks.len());

    let completed_tasks = tasklist.get_tasks(GetTasksFilterOption::Completed);
    assert_eq!(completed_tasks[0].description, "two");
    assert!(completed_tasks.len() == 1);

    let incompleted_tasks = tasklist.get_tasks(GetTasksFilterOption::Incomplete);
    assert_eq!(incompleted_tasks[0].description, "one");
    assert_eq!(incompleted_tasks[1].description, "three");
    assert!(incompleted_tasks.len() == 2);
  }
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
