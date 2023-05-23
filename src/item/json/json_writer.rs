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
    fn write(&mut self, item: &R) -> Result<(), crate::BatchError> {

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

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn open(&mut self) {
        let mut separator = b"[".to_vec();

        if self.pretty_formatter {
            separator.push(b'\n');
        }

        self.writer.write_all(&separator);
    }

    fn update(&mut self, is_first_item: bool) {
        let mut separator = b",".to_vec();

        if self.pretty_formatter {
            separator.push(b'\n');
        }

        if !is_first_item {
            self.writer.write_all(&separator);
        }
    }

    fn close(&mut self) {
        let mut separator = b"]".to_vec();

        if self.pretty_formatter {
            separator = b"\n]".to_vec();
        }

        self.writer.write_all(&separator);
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
