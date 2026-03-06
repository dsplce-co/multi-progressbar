use crate::{visual_len, ProgressBar, TaskProgress};

/// ClassicProgressBar is a classic progress bar, using `[=> ]` to show progress.
pub struct ClassicProgressBar<T: TaskProgress> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ClassicProgressBar<T>
where
    T: TaskProgress,
{
    /// creates a new ClassicProgressBar.
    pub fn new() -> Self {
        ClassicProgressBar {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> ProgressBar for ClassicProgressBar<T>
where
    T: TaskProgress,
{
    type Task = T;
    fn format_line(&self, progress: &T, width: usize) -> String {
        let (before, after) = (progress.before(), progress.after());
        let (current, total) = progress.progress();

        if total == 0 {
            return " ".repeat(width);
        }

        let before_len = before.as_ref().map(|s| visual_len(s)).unwrap_or(0);
        let after_len = after.as_ref().map(|s| visual_len(s)).unwrap_or(0);

        // check if we have enough space to show progress bar
        if before_len + after_len + 2 > width {
            return " ".repeat(width);
        }

        let bar_width = width - before_len - after_len - 2;
        let bar_progress = (current as f64 / total as f64 * bar_width as f64) as usize;

        let mut bar = String::new();
        bar.push_str(&before.unwrap_or_default());
        bar.push('[');
        bar.push_str(&"=".repeat(bar_progress));
        bar.push_str(&" ".repeat(bar_width - bar_progress));
        bar.push(']');
        bar.push_str(&after.unwrap_or_default());
        bar
    }
}
