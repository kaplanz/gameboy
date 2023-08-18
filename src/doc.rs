use std::fs::File;
use std::io::BufWriter;

#[derive(Debug)]
pub struct Doctor {
    pub log: BufWriter<File>,
}

impl Doctor {
    pub fn new(log: File) -> Self {
        Self {
            log: BufWriter::new(log),
        }
    }
}
