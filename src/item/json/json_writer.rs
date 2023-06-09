use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use crate::{core::item::ItemWriter, BatchError};

pub struct JsonItemWriter<'a, T: io::Write> {
    writer: BufWriter<T>,
    indent: &'a [u8],
    pretty_formatter: bool,
}

impl<'a, T: io::Write, R: serde::Serialize> ItemWriter<R> for JsonItemWriter<'a, T> {
    fn write(&mut self, item: &R) -> Result<(), BatchError> {

        let result = if self.pretty_formatter {
            let formatter = serde_json::ser::PrettyFormatter::with_indent(self.indent);
            let mut ser = serde_json::Serializer::with_formatter(&mut self.writer, formatter);
            item.serialize(&mut ser)
        } else {
            let mut ser = serde_json::Serializer::new(&mut self.writer);
            item.serialize(&mut ser)
        };

        match result {
            Ok(_ser) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    fn flush(&mut self) -> Result<(), BatchError> {
        let result = self.writer.flush();

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    fn open(&mut self) -> Result<(), BatchError> {
        let mut separator = b"[".to_vec();

        if self.pretty_formatter {
            separator.push(b'\n');
        }

        let result = self.writer.write_all(&separator);

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }

    fn update(&mut self, is_first_item: bool) -> Result<(), BatchError> {
        let mut separator = b",".to_vec();

        if self.pretty_formatter {
            separator.push(b'\n');
        }

        if !is_first_item {
            let result = self.writer.write_all(&separator);

            return match result {
                Ok(()) => Ok(()),
                Err(error) => Err(BatchError::ItemWriter(error.to_string())),
            };
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), BatchError> {
        let mut separator = b"]".to_vec();

        if self.pretty_formatter {
            separator = b"\n]".to_vec();
        }

        let result = self.writer.write_all(&separator);

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(BatchError::ItemWriter(error.to_string())),
        }
    }
}

#[derive(Default)]
pub struct JsonItemWriterBuilder<'a> {
    file: Option<File>,
    indent: &'a [u8],
    pretty_formatter: bool,
}

impl<'a> JsonItemWriterBuilder<'a> {
    pub fn new() -> JsonItemWriterBuilder<'a> {
        JsonItemWriterBuilder {
            file: None,
            indent: b"  ",
            pretty_formatter: false,
        }
    }

    pub fn file(mut self, file: File) -> JsonItemWriterBuilder<'a> {
        self.file = Some(file);
        self
    }

    pub fn indent(mut self, indent: &'a [u8]) -> JsonItemWriterBuilder<'a> {
        self.indent = indent;
        self
    }

    pub fn pretty_formatter(mut self, yes: bool) -> JsonItemWriterBuilder<'a> {
        self.pretty_formatter = yes;
        self
    }

    pub fn build(self) -> JsonItemWriter<'a, File> {
        let writer = BufWriter::new(self.file.unwrap());

        JsonItemWriter {
            writer,
            indent: self.indent,
            pretty_formatter: self.pretty_formatter,
        }
    }
}
