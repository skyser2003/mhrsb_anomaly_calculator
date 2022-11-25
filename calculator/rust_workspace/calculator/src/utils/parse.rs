use std::{fs::File, io::BufReader, path::Path};

use anyhow::Context;
use serde::de;

pub fn parse_data<T, P: AsRef<Path>>(filename: P) -> Vec<T>
where
    T: de::DeserializeOwned,
{
    File::open(filename)
        .context("Failed to open file")
        .and_then(|file| {
            let reader = BufReader::new(file);

            serde_json::from_reader(reader).context("Failed to parse")
        })
        .unwrap_or_default()
}
