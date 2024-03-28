use std::fmt::Debug;

use log::info;

use crate::core::item::{ItemWriter, ItemWriterResult};

/// A simple item writer that logs the items using the `log` crate.
#[derive(Default)]
pub struct LoggerWriter;

impl<T: Debug> ItemWriter<T> for LoggerWriter {
    /// Writes the items to the log.
    ///
    /// # Arguments
    ///
    /// * `items` - The items to be written.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the items were successfully written to the log.
    ///
    /// # Examples
    ///
    /// ```
    /// use spring_batch_rs::item::logger::LoggerWriter;
    /// use spring_batch_rs::core::item::{ItemWriter, ItemWriterResult};
    ///
    /// let writer = LoggerWriter::default();
    /// let items = vec![1, 2, 3];
    /// let result: ItemWriterResult = writer.write(&items);
    /// assert!(result.is_ok());
    /// ```
    fn write(&self, items: &[T]) -> ItemWriterResult {
        items.iter().for_each(|item| info!("Record:{:?}", item));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write() {
        let writer = LoggerWriter::default();
        let items = vec![1, 2, 3];
        let result = writer.write(&items);
        assert!(result.is_ok());
    }
}
