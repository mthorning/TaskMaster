use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Task {
  pub is_completed: bool,
  pub description: String,
}

#[derive(Debug, PartialEq)]
pub struct TaskList {
  pub tasks: HashMap<String, Task>,
}

const COMPLETE_PREFIX: &str = "- [x] ";
const INCOMPLETE_PREFIX: &str = "- [ ] ";

#[derive(PartialEq)]
pub enum GetTasksFilterOption {
  All,
  Completed,
  Incomplete,
}

impl TaskList {
  pub fn from_string(content: &str) -> TaskList {
    let mut task_list = TaskList {
      tasks: HashMap::new(),
    };

    content.split("\n").for_each(|line| {
      let trimmed = line.trim();

      let mut add_task = |prefix: &str, is_completed: bool| -> bool {
        if trimmed.starts_with(prefix) {
          let description = trimmed.replace(prefix, "");
          task_list.tasks.insert(
            description.clone(),
            Task {
              is_completed,
              description,
            },
          );
          return true;
        }

        false
      };

      if !add_task(COMPLETE_PREFIX, true) {
        add_task(INCOMPLETE_PREFIX, false);
      }
    });

    task_list
  }

  pub fn get_tasks(&self, list_option: GetTasksFilterOption) -> Vec<Task> {
    if GetTasksFilterOption::All == list_option {
      return self.tasks.values().cloned().collect();
    }
    self
      .tasks
      .values()
      .filter(|task| match list_option {
        GetTasksFilterOption::Completed => task.is_completed,
        GetTasksFilterOption::Incomplete => !task.is_completed,
        _ => unreachable!(),
      })
      .cloned()
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_from_string() {
    let test_string = String::from(
      "hello\n- this is a note\n- [ ] incomplete task\n - [x] complete task  \nnothing",
    );
    let result = TaskList::from_string(&test_string);
    let expected = TaskList {
      tasks: HashMap::from([
        (
          "incomplete task".to_string(),
          Task {
            is_completed: false,
            description: "incomplete task".to_string(),
          },
        ),
        (
          "complete task".to_string(),
          Task {
            is_completed: true,
            description: "complete task".to_string(),
          },
        ),
      ]),
    };

    assert_eq!(expected, result);
  }

  #[test]
  fn test_list_tasks() {
    let one = "one".to_string();
    let two = "two".to_string();
    let three = "three".to_string();

    let tasklist = TaskList {
      tasks: HashMap::from([
        (
          one.clone(),
          Task {
            is_completed: false,
            description: one.clone(),
          },
        ),
        (
          two.clone(),
          Task {
            is_completed: true,
            description: two.clone(),
          },
        ),
        (
          three.clone(),
          Task {
            is_completed: false,
            description: three.clone(),
          },
        ),
      ]),
    };

    let all_tasks = tasklist.get_tasks(GetTasksFilterOption::All);
    assert_eq!(tasklist.tasks.len(), all_tasks.len());

    let completed_tasks = tasklist.get_tasks(GetTasksFilterOption::Completed);
    assert_eq!(completed_tasks[0].description, two);
    assert!(completed_tasks.len() == 1);

    let incompleted_tasks = tasklist.get_tasks(GetTasksFilterOption::Incomplete);

    let found_one = incompleted_tasks
      .iter()
      .find(|task| task.description == one);
    assert!(found_one.is_some());

    let found_three = incompleted_tasks
      .iter()
      .find(|task| task.description == three);
    assert!(found_three.is_some());
  }
}
