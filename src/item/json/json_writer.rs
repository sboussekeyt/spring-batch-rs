use std::{
    cell::{Cell, RefCell},
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::{
    core::item::{ItemWriter, ItemWriterResult},
    BatchError,
};

pub struct JsonItemWriter<T: Write> {
    stream: RefCell<BufWriter<T>>,
    use_pretty_formatter: bool,
    is_first_element: Cell<bool>,
}

impl<T: Write, R: serde::Serialize> ItemWriter<R> for JsonItemWriter<T> {
    fn write(&self, items: &[R]) -> ItemWriterResult {
        let mut json_chunk = String::new();

        for item in items.iter() {
            if !self.is_first_element.get() {
                json_chunk.push(',');
            } else {
                self.is_first_element.set(false);
            }

            let result = if self.use_pretty_formatter {
                serde_json::to_string_pretty(item)
            } else {
                serde_json::to_string(item)
            };

            json_chunk.push_str(&result.unwrap());

            if self.use_pretty_formatter {
                json_chunk.push('\n');
            }
        }

        let result = self.stream.borrow_mut().write_all(json_chunk.as_bytes());

        match result {
            Ok(_ser) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    fn flush(&self) -> ItemWriterResult {
        let result = self.stream.borrow_mut().flush();

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    fn open(&self) -> ItemWriterResult {
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

    fn close(&self) -> ItemWriterResult {
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
    pub fn new() -> Self {
        Self {
            indent: Box::from(b"  ".to_vec()),
            pretty_formatter: false,
        }
    }

    pub fn indent(mut self, indent: &[u8]) -> Self {
        self.indent = Box::from(indent);
        self
    }

    pub fn pretty_formatter(mut self, yes: bool) -> Self {
        self.pretty_formatter = yes;
        self
    }

    pub fn from_path<R: AsRef<Path>>(self, path: R) -> JsonItemWriter<File> {
        let file = File::create(path).expect("Unable to open file");

        let buf_writer = BufWriter::new(file);

        JsonItemWriter {
            stream: RefCell::new(buf_writer),
            use_pretty_formatter: self.pretty_formatter,
            is_first_element: Cell::new(true),
        }
    }

    pub fn from_writer<W: Write>(self, wtr: W) -> JsonItemWriter<W> {
        let buf_writer = BufWriter::new(wtr);

        JsonItemWriter {
            stream: RefCell::new(buf_writer),
            use_pretty_formatter: self.pretty_formatter,
            is_first_element: Cell::new(true),
        }
    }
}
