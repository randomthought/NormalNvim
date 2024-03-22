use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    pin::Pin,
};

use color_eyre::eyre::Result;
use futures_util::{stream, Stream};

pub fn create_stream(
    path: &Path,
    buf_size: usize,
) -> Result<Pin<Box<dyn Stream<Item = eyre::Result<String>> + Sync + Send>>> {
    // TODO: You can use chain for multiple files https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.chain
    let file = File::options().write(false).read(true).open(path)?;
    let reader = BufReader::with_capacity(buf_size, file);

    let fs = reader.lines().map(|line| line.map_err(eyre::Error::from));

    let stream = stream::iter(fs);
    Ok(Box::pin(stream))
}
