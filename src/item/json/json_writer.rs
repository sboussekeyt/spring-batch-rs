use std::{
    cell::RefCell,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::{core::item::ItemWriter, BatchError};

pub struct JsonItemWriter<T: Write> {
    stream: RefCell<BufWriter<T>>,
    use_pretty_formatter: bool,
}

impl<T: Write, R: serde::Serialize> ItemWriter<R> for JsonItemWriter<T> {
    fn write(&self, item: &R) -> Result<(), BatchError> {
        let json = if self.use_pretty_formatter {
            serde_json::to_string_pretty(item)
        } else {
            serde_json::to_string(item)
        };
        let result = self.stream.borrow_mut().write_all(json.unwrap().as_bytes());

        match result {
            Ok(_ser) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    fn flush(&self) -> Result<(), BatchError> {
        let result = self.stream.borrow_mut().flush();

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    fn open(&self) -> Result<(), BatchError> {
        let begin_array = if self.use_pretty_formatter {
            b"[\n".to_vec()
        } else {
            b"[".to_vec()
        };

        let result = self.stream.borrow_mut().write_all(&begin_array);

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    fn update(&self, is_first_item: bool) -> Result<(), BatchError> {
        if !is_first_item {
            let separator = if self.use_pretty_formatter {
                b",\n".to_vec()
            } else {
                b",".to_vec()
            };

            let result = self.stream.borrow_mut().write_all(&separator);

            return match result {
                Ok(()) => Ok(()),
                Err(error) => Err(BatchError::ItemWriter(error.to_string())),
            };
        }
        Ok(())
    }

    fn close(&self) -> Result<(), BatchError> {
        let end_array = if self.use_pretty_formatter {
            b"\n]\n".to_vec()
        } else {
            b"]\n".to_vec()
        };

        let result = self.stream.borrow_mut().write_all(&end_array);
        let _ = self.stream.borrow_mut().flush();

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }
}

#[derive(Default)]
pub struct JsonItemWriterBuilder {
    indent: Box<[u8]>,
    pretty_formatter: bool,
}

impl JsonItemWriterBuilder {
    pub fn new() -> JsonItemWriterBuilder {
        JsonItemWriterBuilder {
            indent: Box::from(b"  ".to_vec()),
            pretty_formatter: false,
        }
    }

    pub fn indent(mut self, indent: &[u8]) -> JsonItemWriterBuilder {
        self.indent = Box::from(indent);
        self
    }

    pub fn pretty_formatter(mut self, yes: bool) -> JsonItemWriterBuilder {
        self.pretty_formatter = yes;
        self
    }

    pub fn from_path<R: AsRef<Path>>(self, path: R) -> JsonItemWriter<File> {
        let file = File::create(path).expect("Unable to open file");

        let buf_writer = BufWriter::new(file);

        JsonItemWriter {
            stream: RefCell::new(buf_writer),
            use_pretty_formatter: self.pretty_formatter,
        }
    }
}
