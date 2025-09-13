//! vocage — modularized

mod format;
pub mod model;

pub use crate::format::PrintFormat;
pub use crate::model::{VocaCard, VocaData, VocaSession};

use std::path::PathBuf;

/// Load multiple files into datasets, checking column compatibility unless `force == true`.
/// Mirrors your original `load_files(files, force, reset)` API.
pub fn load_files(files: Vec<&str>, force: bool, reset: bool) -> Vec<VocaData> {
    let mut datasets: Vec<VocaData> = Vec::new();

    for filename in files.iter() {
        if !PathBuf::from(filename).exists() {
            eprintln!("ERROR: Specified input file does not exist: {}", filename);
            std::process::exit(1);
        }

        match VocaData::from_file(filename, reset) {
            Ok(data) => {
                if let Some(first) = datasets.first() {
                    if !force && data.session.columns != first.session.columns {
                        eprintln!(
                            "ERROR: columns of {} differ from those in the first loaded file; \
                             use --force to bypass.",
                            filename
                        );
                        std::process::exit(1);
                    }
                }
                datasets.push(data);
            }
            Err(err) => {
                eprintln!("ERROR loading {}: {}", filename, err);
                std::process::exit(1);
            }
        }
    }

    datasets
}
