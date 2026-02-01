use std::{cmp::Ordering, sync::Arc};

#[derive(Debug, PartialEq, Eq)]
pub struct Task {
  pub is_completed: bool,
  pub description: String,
}

#[derive(Debug, PartialEq)]
pub enum HashMapTaskType {
  Existing,
  Added,
  Deleted,
}

#[derive(Debug, PartialEq)]
struct OriginalTask {
  is_completed: bool,
  description: Arc<str>,
}

#[derive(Debug)]
pub struct HashMapTask {
  is_completed: bool,
  description: Arc<str>,
  original: OriginalTask,
  order: usize,
  pub task_type: HashMapTaskType,
}

impl HashMapTask {
  pub fn from(task: Task, order: usize) -> HashMapTask {
    let desc_arc: Arc<str> = Arc::from(task.description);
    HashMapTask {
      order,
      original: OriginalTask {
        is_completed: task.is_completed,
        description: desc_arc.clone(),
      },
      is_completed: task.is_completed,
      description: desc_arc,
      task_type: HashMapTaskType::Existing,
    }
  }

  pub fn new(description: String, order: usize) -> HashMapTask {
    let desc_arc: Arc<str> = Arc::from(description);
    HashMapTask {
      order,
      original: OriginalTask {
        is_completed: false,
        description: desc_arc.clone(),
      },
      is_completed: false,
      description: desc_arc,
      task_type: HashMapTaskType::Added,
    }
  }

  pub fn get_task(&self) -> Task {
    Task {
      is_completed: self.is_completed,
      description: self.description.to_string(),
    }
  }

  pub fn get_original_task(&self) -> Task {
    Task {
      is_completed: self.original.is_completed,
      description: self.original.description.to_string(),
    }
  }

  pub fn get_key(&self) -> Arc<str> {
    self.description.clone()
  }

  pub fn get_original_key(&self) -> Arc<str> {
    self.original.description.clone()
  }

  pub fn toggle(&mut self) {
    self.is_completed = !self.is_completed;
  }

  pub fn set_description(&mut self, new_description: &str) -> Arc<str> {
    self.description = Arc::from(new_description);
    self.description.clone()
  }

  pub fn delete(&mut self) {
    self.task_type = HashMapTaskType::Deleted;
  }
}

impl Eq for HashMapTask {}

impl PartialEq for HashMapTask {
  fn eq(&self, other: &Self) -> bool {
    self.order == other.order
  }
}

impl PartialEq<HashMapTaskType> for HashMapTask {
  fn eq(&self, other: &HashMapTaskType) -> bool {
    self.task_type == *other
  }
}

impl PartialOrd for HashMapTask {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for HashMapTask {
  fn cmp(&self, other: &Self) -> Ordering {
    self.order.cmp(&other.order)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_task_equality_for_toggle() {
    let mut hmt = HashMapTask::from(Task {
      is_completed: false,
      description: "first desc".to_string(),
    }, 0);

    assert_eq!(hmt.get_task(), hmt.get_original_task());

    hmt.toggle();
    assert_ne!(hmt.get_task(), hmt.get_original_task());

    hmt.toggle();
    assert_eq!(hmt.get_task(), hmt.get_original_task());
  }

  #[test]
  fn test_task_equality_for_description() {
    let mut hmt = HashMapTask::from(Task {
      is_completed: false,
      description: "first desc".to_string(),
    }, 0);

    assert_eq!(hmt.get_task(), hmt.get_original_task());

    let _ = hmt.set_description("new desc");
    assert_ne!(hmt.get_task(), hmt.get_original_task());

    let _ = hmt.set_description("first desc");
    assert_eq!(hmt.get_task(), hmt.get_original_task());
  }
}
