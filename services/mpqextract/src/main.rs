use std::{ffi::OsStr, io::BufRead, path::PathBuf};

use clap::{Parser, Subcommand};
use mpq::Archive;
use rayon::prelude::*;

/// Extract data from MPQ files
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    #[clap(subcommand)]
    command: Command,

    /// Plain output (useful for other tools)
    #[clap(short)]
    plain: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List the (merged) files and exit
    List {
        /// Paths to ignore
        #[clap(short)]
        exclude: Vec<String>,

        /// Paths to include
        #[clap(short)]
        include: Vec<String>,

        paths: Vec<PathBuf>,
    },
    /// Extract the files to a destination.
    Extract {
        /// Paths to ignore
        #[clap(short)]
        exclude: Vec<String>,

        /// Paths to include
        #[clap(short)]
        include: Vec<String>,

        /// The destination folder to extract into.
        dest: PathBuf,

        /// The mpq files to extract.
        paths: Vec<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    match opts.command {
        Command::List {
            paths,
            exclude,
            include,
        } => {
            for path in paths {
                let mut header_written = false;

                let mut a = Archive::open(&path).unwrap();
                let file = a.open_file("(listfile)")?;

                let mut buf: Vec<u8> = vec![0; file.size() as usize];
                file.read(&mut a, &mut buf).unwrap();
                let string = std::str::from_utf8(&buf)?;

                for file in string
                    .lines()
                    // .filter(|p| exclude.iter().any(|test| !p.contains(test)))
                    .filter(|p| {
                        if include.is_empty() {
                            true
                        } else {
                            include.iter().all(|test| p.contains(test))
                        }
                    })
                {
                    if !header_written {
                        if let Some(name) = path.file_name().and_then(OsStr::to_str) {
                            println!("\u{001b}[32m{}\u{001b}[0m", name)
                        }
                        header_written = true;
                    }

                    println!("\u{001b}[30m- {}\u{001b}[0m", file);
                }
            }
        }
        Command::Extract {
            paths,
            dest,
            exclude,
            include,
        } => {
            paths
                .par_iter()
                .map(|path| {
                    let mut header_written = false;

                    let mut a = Archive::open(&path)?;

                    let buf = {
                        let file = a.open_file("(listfile)")?;
                        let mut buf: Vec<u8> = vec![0; file.size() as usize];
                        file.read(&mut a, &mut buf).unwrap();
                        buf
                    };

                    let string = std::str::from_utf8(&buf)?;
                    for file_name in string
                        .lines()
                        // .filter(|p| exclude.iter().any(|test| !p.contains(test)))
                        .filter(|p| {
                            if include.is_empty() {
                                true
                            } else {
                                include.iter().all(|test| p.contains(test))
                            }
                        })
                    {
                        println!("extracting {}", file_name);
                        let file = a.open_file(file_name)?;
                        file.extract(&mut a, &dest.join(file_name.replace("\\", "/")));
                    }

                    Ok(())
                })
                .collect::<anyhow::Result<Vec<_>>>();
        }
    }

    Ok(())
}
