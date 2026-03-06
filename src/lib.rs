//! # multi-progress
//!
//! multi-progress is a library to show multiple progress bars along with log outputs in terminal.
//!
//! ## Usage
//!
//! 1. Implement [TaskProgress] trait for your task.
//! 2. Call [MultiProgressBar::new] with a [ProgressBar] implementation (provided in the [bar] module).
//! 3. Call [MultiProgressBar::draw] to draw progress bars when needed.
//!
//! ```rust
//! use multi_progressbar::{
//!     MultiProgressBar, TaskProgress,
//!     bar::classic::ClassicProgressBar
//! };
//!
//! struct Task {
//!     name: String,
//!     progress: u64,
//!     total: u64,
//! }
//!
//! impl TaskProgress for Task {
//!     fn progress(&self) -> (u64, u64) {
//!         (self.progress, self.total)
//!     }
//!     fn after(&self) -> Option<String> {
//!         Some(format!("{}/{} completed", self.progress, self.total))
//!     }
//!     fn before(&self) -> Option<String> {
//!         Some(self.name.clone())
//!     }
//! }
//!
//! let mp = MultiProgressBar::new(ClassicProgressBar::new());
//! let task1 = Task {
//!    name: "task1".to_string(),
//!    progress: 0,
//!    total: 100,
//! };
//! let task2 = Task {
//!     name: "task2".to_string(),
//!     progress: 33,
//!     total: 100,
//! };
//! let tasks = vec![task1, task2];
//! mp.draw(&tasks).unwrap();
//!
//!
//! ```

#![warn(missing_docs)]

use crossterm::{cursor, queue, style, terminal};
use std::{
    io::Write,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

/// bar module contains premade progress bar styles.
pub mod bar;

/// Calculates the visual length of a string, excluding ANSI escape sequences.
/// This is useful for calculating the actual display width of strings that contain
/// ANSI color codes or other terminal escape sequences.
pub fn visual_len(s: &str) -> usize {
    let mut len = 0;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip ANSI escape sequence
            // Format is typically ESC [ ... m
            if chars.as_str().starts_with('[') {
                chars.next(); // skip '['
                              // Skip until we find 'm' or reach end
                while let Some(ch) = chars.next() {
                    if ch == 'm' {
                        break;
                    }
                }
            }
        } else {
            len += 1;
        }
    }
    len
}

/// Task is abstraction for one single running task.
pub trait TaskProgress {
    /// returns the current progress and total progress.
    fn progress(&self) -> (u64, u64);
    /// returns message to show before progress bar
    fn before(&self) -> Option<String> {
        None
    }
    /// returns message to show after progress bar
    fn after(&self) -> Option<String> {
        None
    }
}

/// ProgressBar is an abstraction for the appearance of a progress bar.
pub trait ProgressBar {
    /// Progress is provided by TaskProgress.
    type Task: TaskProgress;
    /// formats a line of progress bar to show in terminal.
    fn format_line(&self, progress: &Self::Task, width: usize) -> String;
}

/// MultiProgress is the main struct of this library.
/// It handles drawing progress bars and log outputs.
pub struct MultiProgressBar<P: ProgressBar> {
    progress_bar: P,
    tasks: Arc<Mutex<Vec<P::Task>>>,
    starting_y: AtomicUsize,
}

impl<P: ProgressBar> MultiProgressBar<P> {
    /// creates a new MultiProgress with given ProgressBar style.
    pub fn new(progress_bar: P, tasks: Arc<Mutex<Vec<P::Task>>>) -> Self {
        MultiProgressBar {
            progress_bar,
            tasks,
            starting_y: AtomicUsize::new(0),
        }
    }

    fn starting_y(&self) -> usize {
        self.starting_y.load(Ordering::Relaxed)
    }

    /// logs a message above progress bars.
    pub fn log(&self, msg: &str) -> std::io::Result<()> {
        let starting_y = self.starting_y();

        if starting_y == 0 {
            // Not initialized yet, just print normally
            println!("{}", msg);
            return Ok(());
        }

        let (width, _) = crossterm::terminal::size().unwrap();
        let mut stdout = std::io::stdout();

        queue!(
            stdout,
            cursor::MoveToRow((starting_y - 1) as u16),
            cursor::MoveToColumn(0),
        )?;

        write!(stdout, "{:width$}", msg, width = width as usize)?;
        stdout.flush()
    }

    /// draws the progress bars.
    pub fn draw(&self) -> std::io::Result<()> {
        // Initialize starting_y on first call
        if self.starting_y() == 0 {
            let (_, y) = crossterm::cursor::position().unwrap();
            self.starting_y.store(y as usize + 1, Ordering::Relaxed);
        }

        let starting_y = self.starting_y();
        let tasks_no = {
            let tasks = self.tasks.lock().unwrap();
            tasks.len()
        };

        if tasks_no == 0 {
            return Ok(());
        }

        let (width, height) = crossterm::terminal::size().unwrap();
        let mut stdout = std::io::stdout();

        // Ensure we have enough space by printing newlines if needed
        let max_row = height as usize - 1;
        let last_row_needed = starting_y + tasks_no - 1;

        if last_row_needed > max_row {
            // We need more space - print newlines to create it
            let lines_to_add = last_row_needed - max_row;
            queue!(
                stdout,
                cursor::MoveToRow(max_row as u16),
                cursor::MoveToColumn(0)
            )?;
            for _ in 0..lines_to_add {
                queue!(stdout, style::Print("\n"))?;
            }
            stdout.flush()?;

            // Adjust starting_y since everything scrolled up
            self.starting_y
                .store(starting_y - lines_to_add, Ordering::Relaxed);
        }

        let starting_y = self.starting_y();

        // Draw all progress bars
        queue!(
            stdout,
            terminal::BeginSynchronizedUpdate,
            cursor::MoveToRow(starting_y as u16),
            cursor::MoveToColumn(0)
        )?;

        let tasks = self.tasks.lock().unwrap();

        for task in tasks.iter() {
            let line = self.progress_bar.format_line(task, width as usize);
            write!(stdout, "{}", line)?;
            queue!(stdout, cursor::MoveToColumn(0), cursor::MoveDown(1))?;
        }

        drop(tasks);

        queue!(
            stdout,
            terminal::EndSynchronizedUpdate,
            cursor::MoveToColumn(0),
        )?;

        stdout.flush()?;

        Ok(())
    }
}
