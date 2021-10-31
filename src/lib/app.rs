use lazy_static::lazy_static;
use std::sync::RwLock;

#[derive(Debug)]
pub struct App {
  hm_task_output: Vec<String>,
  hm_task_summary: Vec<String>,
}

impl App {
  pub fn append_output(&mut self, output: String) {
    self.hm_task_output.push(output);
  }

  pub fn append_summary(&mut self, summary: String) {
    self.hm_task_summary.push(summary);
  }

  /// Set the app's hm task summary.
  pub fn set_hm_task_summary(&mut self, hm_task_summary: Vec<String>) {
    self.hm_task_summary = hm_task_summary;
  }

  /// Set the app's hm task output.
  pub fn set_hm_task_output(&mut self, hm_task_output: Vec<String>) {
    self.hm_task_output = hm_task_output;
  }

  /// Get a reference to the app's hm task output.
  pub fn hm_task_output(&self) -> Vec<String> {
    self.hm_task_output.clone()
  }

  /// Get a reference to the app's hm task summary.
  pub fn hm_task_summary(&self) -> Vec<String> {
    self.hm_task_summary.clone()
  }
}

impl Default for App {
  fn default() -> App {
    App {
      hm_task_output: Vec::new(),
      hm_task_summary: Vec::new(),
    }
  }
}

lazy_static! {
  pub static ref APP: RwLock<App> = RwLock::new(App::default());
}

fn tui_element_append_output(output: String) {
  APP.write().unwrap().append_output(output);
}

fn tui_element_append_summary(summary: String) {
  APP.write().unwrap().append_summary(summary);
}
