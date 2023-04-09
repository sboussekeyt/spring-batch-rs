use std::{
    cell::RefCell,
    fs::File,
    io::{BufWriter, Write},
};

use crate::{core::item::ItemWriter, BatchError};

pub struct JsonItemWriter {
    stream: RefCell<BufWriter<File>>,
    use_pretty_formatter: bool,
}

impl JsonItemWriter {
    pub fn new(path: &str, use_pretty_formatter: bool) -> Self {
        let file = File::options()
            .append(true)
            .read(false)
            .create(true)
            .open(path)
            .expect("Unable to open file");

        let buf_writer = BufWriter::new(file);

        Self {
            stream: RefCell::new(buf_writer),
            use_pretty_formatter,
        }
    }
}

impl<R: serde::Serialize> ItemWriter<R> for JsonItemWriter {
    fn write(&self, item: &R) -> Result<(), BatchError> {
        let json = if self.use_pretty_formatter {
            serde_json::to_string_pretty(item)
        } else {
            serde_json::to_string(item)
        };
        let result = self.stream.borrow_mut().write_all(json.unwrap().as_bytes());

        let _ = match result {
            Ok(_ser) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        };

        Ok(())
    }

    fn flush(&self) -> Result<(), BatchError> {
        let result = self.stream.borrow_mut().flush();

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    fn open(&self) -> Result<(), BatchError> {
        let mut separator = vec![b'['; 2];

        if self.use_pretty_formatter {
            separator.push(b'\n');
        }

        let result = self.stream.borrow_mut().write_all(&separator);

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    fn update(&self, is_first_item: bool) -> Result<(), BatchError> {
        let mut separator = vec![b','; 2];

        if self.use_pretty_formatter {
            separator.push(b'\n');
        }

        if !is_first_item {
            let result = self.stream.borrow_mut().write_all(&separator);

            return match result {
                Ok(()) => Ok(()),
                Err(error) => Err(BatchError::ItemWriter(error.to_string())),
            };
        }
        Ok(())
    }

    fn close(&self) -> Result<(), BatchError> {
        let mut separator = vec![b']'; 2];

        if self.use_pretty_formatter {
            separator = vec![b'\n', b']'];
        }

        let result = self.stream.borrow_mut().write_all(&separator);

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }
}

#[derive(Default)]
pub struct JsonItemWriterBuilder<'a> {
    path: Option<&'a str>,
    indent: Box<[u8]>,
    pretty_formatter: bool,
}

impl<'a> JsonItemWriterBuilder<'a> {
    pub fn new() -> JsonItemWriterBuilder<'a> {
        JsonItemWriterBuilder {
            path: None,
            indent: Box::from(b"  ".to_vec()),
            pretty_formatter: false,
        }
    }

    pub fn path(mut self, path: Option<&'a str>) -> JsonItemWriterBuilder {
        self.path = path;
        self
    }

    pub fn indent(mut self, indent: &'a [u8]) -> JsonItemWriterBuilder {
        self.indent = Box::from(indent);
        self
    }

    pub fn pretty_formatter(mut self, yes: bool) -> JsonItemWriterBuilder<'a> {
        self.pretty_formatter = yes;
        self
    }

    pub fn build(self) -> JsonItemWriter {
        JsonItemWriter::new(self.path.unwrap(), self.pretty_formatter)
    }
}
