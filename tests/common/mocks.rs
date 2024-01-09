//! Mock version of std::fs::File;
use mockall::mock;

use std::io::{self, Write};

mock! {
    pub File {}
    impl Write for File {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
        fn flush(&mut self) -> io::Result<()>;
    }
}
