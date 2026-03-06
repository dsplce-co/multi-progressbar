use crate::{visual_len, ProgressBar, TaskProgress};

/// BlockProgressBar is a progress bar using block characters to show progress.
pub struct BlockProgressBar<T: TaskProgress> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> BlockProgressBar<T>
where
    T: TaskProgress,
{
    /// creates a new BlockProgressBar.
    pub fn new() -> Self {
        BlockProgressBar {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> BlockProgressBar<T>
where
    T: TaskProgress,
{
    fn block_char(width: usize, progress: f64) -> String {
        let block_chars = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
        let progress_in_block = progress * width as f64;
        let complete_blocks = progress_in_block.floor() as usize;
        let remainder = progress_in_block - complete_blocks as f64;
        let mut bar = String::new();
        bar.push_str(&"█".repeat(complete_blocks));
        if remainder > 0.0 {
            let remainder_index = (remainder * 8.0).floor() as usize;
            bar.push(block_chars[remainder_index]);
        }
        bar.push_str(&" ".repeat(width - complete_blocks));
        bar
    }
}

impl<T> ProgressBar for BlockProgressBar<T>
where
    T: TaskProgress,
{
    type Task = T;
    fn format_line(&self, progress: &Self::Task, width: usize) -> String {
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
        let bar_progress = current as f64 / total as f64;

        let mut bar = String::new();
        bar.push_str(&before.unwrap_or_default());
        bar.push('[');
        bar.push_str(&Self::block_char(bar_width, bar_progress));
        bar.push(']');
        bar.push_str(&after.unwrap_or_default());
        bar
    }
}
