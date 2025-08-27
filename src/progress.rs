use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Unified progress reporting system
#[derive(Clone)]
pub struct ProgressReporter {
    progress_bar: Option<ProgressBar>,
    enable_progress: bool,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(enable_progress: bool) -> Self {
        Self {
            progress_bar: None,
            enable_progress,
        }
    }

    /// Create a spinner progress bar
    pub fn create_spinner(&mut self, message: &str) -> ProgressBar {
        if !self.enable_progress {
            return ProgressBar::hidden();
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));

        self.progress_bar = Some(pb.clone());
        pb
    }

    /// Create a progress bar with total
    pub fn create_progress_bar(&mut self, total: u64, message: &str) -> ProgressBar {
        if !self.enable_progress {
            return ProgressBar::hidden();
        }

        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) @ {bytes_per_sec}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_message(message.to_string());

        self.progress_bar = Some(pb.clone());
        pb
    }

    /// Update progress message
    pub fn set_message(&self, message: &str) {
        if let Some(pb) = &self.progress_bar {
            pb.set_message(message.to_string());
        }
    }

    /// Update progress position
    pub fn set_position(&self, pos: u64) {
        if let Some(pb) = &self.progress_bar {
            pb.set_position(pos);
        }
    }

    /// Increment progress
    pub fn inc(&self, delta: u64) {
        if let Some(pb) = &self.progress_bar {
            pb.inc(delta);
        }
    }

    /// Finish progress with success message
    pub fn finish_with_success(&self, message: &str) {
        if let Some(pb) = &self.progress_bar {
            pb.finish_with_message(format!("✅ {message}"));
        }
    }

    /// Finish progress with error message
    pub fn finish_with_error(&self, message: &str) {
        if let Some(pb) = &self.progress_bar {
            pb.finish_with_message(format!("❌ {message}"));
        }
    }

    /// Clear progress bar
    pub fn clear(&self) {
        if let Some(pb) = &self.progress_bar {
            pb.finish_and_clear();
        }
    }

    /// Check if progress is enabled
    pub fn is_enabled(&self) -> bool {
        self.enable_progress
    }
}

/// Progress steps for multi-step operations
pub struct ProgressSteps {
    current: usize,
    total: usize,
    reporter: ProgressReporter,
}

impl ProgressSteps {
    /// Create new progress steps
    pub fn new(total: usize, enable_progress: bool) -> Self {
        Self {
            current: 0,
            total,
            reporter: ProgressReporter::new(enable_progress),
        }
    }

    /// Advance to next step
    pub fn next_step(&mut self, message: &str) {
        self.current += 1;
        let step_msg = format!("({}/{}) {}", self.current, self.total, message);
        self.reporter.create_spinner(&step_msg);
    }

    /// Finish all steps
    pub fn finish(&self) {
        self.reporter.clear();
    }
}
