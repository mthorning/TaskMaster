#[cfg(test)]
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
  let expected = TaskList::from(vec![
    Task {
      is_completed: false,
      description: String::from("incomplete task"),
    },
    Task {
      is_completed: true,
      description: String::from("complete task"),
    },
  ]);

  assert_eq!(expected, result.unwrap());
}

#[test]
fn test_add_task() {
  let mut tasklist = TaskList::from(Vec::new());

  if let Err(err) = tasklist.add_task(&String::from("test description")) {
    panic!("{}", err);
  }

  let task = tasklist
    .tasks
    .get("test description")
    .unwrap_or_else(|| panic!("Task is None"));

  assert_eq!("test description", &*task.description);
  assert!(task.is_completed == false);
  assert_eq!(0, task.order);
  assert_eq!(1, tasklist.tasks.len());
}

#[test]
fn test_list_tasks() {
  let tasklist = TaskList::from(vec![
    Task {
      is_completed: false,
      description: String::from("one"),
    },
    Task {
      is_completed: true,
      description: String::from("two"),
    },
    Task {
      is_completed: false,
      description: String::from("three"),
    },
  ]);

  let all_tasks = tasklist.get_tasks(GetTasksFilterOption::All);
  assert_eq!(all_tasks.len(), tasklist.tasks.len());

  let completed_tasks = tasklist.get_tasks(GetTasksFilterOption::Completed);
  assert_eq!("two", &*completed_tasks[0].description);
  assert!(completed_tasks.len() == 1);

  let incompleted_tasks = tasklist.get_tasks(GetTasksFilterOption::Incomplete);
  assert_eq!("one", &*incompleted_tasks[0].description);
  assert_eq!("three", &*incompleted_tasks[1].description);
  assert!(incompleted_tasks.len() == 2);
}

#[test]
fn test_to_markdown() {
  let mut tasklist = TaskList::from(vec![
    Task {
      is_completed: true,
      description: String::from("one"),
    },
    Task {
      is_completed: false,
      description: String::from("two"),
    },
  ]);

  let orig_one = tasklist.tasks.get("one").unwrap();
  tasklist.tasks.insert(
    Arc::from("one"),
    HashMapTask {
      description: Arc::from("updated task"),
      ..*orig_one
    },
  );

  let orig_two = tasklist.tasks.get("two").unwrap();
  tasklist.tasks.insert(
    Arc::from("two"),
    HashMapTask {
      description: Arc::from("another updated task"),
      ..*orig_two
    },
  );

  tasklist
    .add_task(&String::from("a whole new task"))
    .unwrap();

  let mut test_lines = vec![
    String::from("hello"),
    String::from("- this is a note"),
    String::from("- [ ] one"),
    String::from("- [x] two"),
    String::from("nothing"),
  ];

  let result = tasklist.to_markdown(&mut test_lines);
  assert!(result.is_ok());

  let expected = vec![
    String::from("hello"),
    String::from("- this is a note"),
    String::from("- [x] updated task"),
    String::from("- [ ] another updated task"),
    String::from("nothing"),
    String::from("- [ ] a whole new task"),
  ];

  assert_eq!(expected, test_lines);
}

#[test]
fn test_update_task() {
  let mut tasklist = TaskList::from(vec![
    Task {
      description: String::from("task to toggle"),
      is_completed: true,
    },
    Task {
      description: String::from("task to delete"),
      is_completed: false,
    },
    Task {
      description: String::from("task to edit"),
      is_completed: true,
    },
  ]);

  tasklist.update_task(&TaskUpdateAction::Toggle, &String::from("task to toggle"));
  tasklist.update_task(&TaskUpdateAction::Delete, &String::from("task to delete"));

  let mut lines = vec![
    String::from("- [x] task to toggle"),
    String::from("- [ ] task to delete"),
  ];
  tasklist.to_markdown(&mut lines).unwrap();
  assert_eq!(vec![String::from("- [ ] task to toggle"),], lines);
}

#[test]
fn test_find_partial_matches() {
  let tasklist = TaskList::from(vec![
    Task {
      is_completed: true,
      description: String::from("This is a wibbly task"),
    },
    Task {
      is_completed: false,
      description: String::from("Another wibbly task"),
    },
    Task {
      is_completed: false,
      description: String::from("This is a wobbly task"),
    },
  ]);

  assert_eq!(
    2,
    tasklist
      .find_by_desc("wibbly", GetTasksFilterOption::All)
      .len()
  );
  assert_eq!(
    1,
    tasklist
      .find_by_desc("wibbly", GetTasksFilterOption::Completed)
      .len()
  );
  assert_eq!(
    1,
    tasklist
      .find_by_desc("wobbly", GetTasksFilterOption::All)
      .len()
  );
  assert_eq!(
    2,
    tasklist
      .find_by_desc("this", GetTasksFilterOption::All)
      .len()
  );
}
