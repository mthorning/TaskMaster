use anyhow::Result;
use std::fs::{OpenOptions, metadata};
use std::io::{Read, Write};

pub struct File<'a> {
  pub path: &'a str,
}

impl<'a> File<'a> {
  pub fn from(path: &'a str) -> File<'a> {
    File { path }
  }

  pub fn append_to_file(&self, content: &str) -> Result<()> {
    let needs_newline = match metadata(&self.path) {
      Ok(meta) => meta.len() > 0,
      Err(_) => false,
    };

    let mut file = OpenOptions::new()
      .append(true)
      .create(true)
      .open(&self.path)?;

    let new_line = if needs_newline { "\n" } else { "" };
    let content_str = format!("{}{}", new_line, content);

    file.write_all(content_str.as_bytes())?;

    Ok(())
  }

  pub fn get_contents(&self) -> Result<String> {
    let mut file = OpenOptions::new().read(true).open(&self.path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
  }
}
