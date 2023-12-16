use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
    path::Path,
    pin::Pin,
};

use anyhow::Result;
use futures_util::{stream, Stream};

struct FileStream {
    reader: Lines<BufReader<File>>,
}

impl Iterator for FileStream {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader
            .next()
            .map(|line| line.map_err(anyhow::Error::from))
    }
}

pub fn create_stream(path: &Path) -> Result<Pin<Box<dyn Stream<Item = Result<String>>>>> {
    // TODO: You can use chain for multiple files https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.chain
    let file = File::options().write(false).read(true).open(path)?;
    let reader = BufReader::new(file);
    let fs = FileStream {
        reader: reader.lines(),
    };

    let stream = stream::iter(fs);
    let pin = Box::pin(stream);

    Ok(pin)
}
