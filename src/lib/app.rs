use lazy_static::lazy_static;
use std::sync::RwLock;

pub(crate) struct App {
  hm_task_output: Vec<String>,
  hm_task_summary: Vec<String>,
}

impl App {
  pub(crate) fn append_output(&mut self, output: String) {
    self.hm_task_output.push(output);
  }

  pub(crate) fn append_summary(&mut self, summary: String) {
    self.hm_task_summary.push(summary);
  }

  /// Set the app's hm task summary.
  pub(crate) fn set_hm_task_summary(&mut self, hm_task_summary: Vec<String>) {
    self.hm_task_summary = hm_task_summary;
  }

  /// Set the app's hm task output.
  pub(crate) fn set_hm_task_output(&mut self, hm_task_output: Vec<String>) {
    self.hm_task_output = hm_task_output;
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
  static ref APP: RwLock<App> = RwLock::new(App::default());
}

pub fn tui_element_append_output(output: String) {
  APP.write().unwrap().append_output(output);
}

pub fn tui_element_append_summary(summary: String) {
  APP.write().unwrap().append_summary(summary);
}
