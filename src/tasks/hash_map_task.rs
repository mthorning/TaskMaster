use crate::tasks::Task;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub struct OriginalTask {
  pub is_completed: bool,
  pub description: Arc<str>,
}

#[derive(Debug, PartialEq)]
pub struct HashMapTask {
  pub original: OriginalTask,
  pub order: usize,
  pub is_completed: bool,
  pub description: Arc<str>,
}

impl HashMapTask {
  pub fn from(task: Task, order: usize) -> HashMapTask {
    let desc_arc: Arc<str> = Arc::from(task.description);
    HashMapTask {
      original: OriginalTask {
        is_completed: task.is_completed,
        description: desc_arc.clone(),
      },
      is_completed: task.is_completed,
      description: desc_arc,
      order,
    }
  }
}
