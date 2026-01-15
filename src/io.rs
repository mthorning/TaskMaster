use std::io::{self, Write};

pub fn prompt_user(prompt: &str) -> String {
  println!("{}", prompt);
  if let Err(err) = io::stdout().flush() {
    eprintln!("{}", err);
  }
  let mut answer = String::new();
  if let Err(err) = io::stdin().read_line(&mut answer) {
    eprintln!("{}", err);
  }

  answer.trim().to_owned()
}
