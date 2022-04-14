use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    borrow::Cow,
    error::Error,
    fs::{create_dir, metadata, read_dir, File},
    io::copy,
    path::{Path, PathBuf},
    process::exit,
};
use zip::ZipArchive;

#[derive(Parser, Debug)]
/// Image file extractor from ePub file
struct App {
    /// epub or zip file paths
    files: Vec<String>,
    /// Use full name for output directory name
    #[clap(short, long)]
    fullname: bool,
}

fn main() {
    if let Err(e) = main2() {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

fn main2() -> Result<(), String> {
    let args = App::parse();

    let files = args
        .files
        .into_iter()
        .map(|p| {
            metadata(&p).and_then(|m| {
                if m.is_dir() {
                    read_dir(&p).map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect())
                } else {
                    Ok(vec![PathBuf::from(p)])
                }
            })
        })
        .filter_map(|r| r.ok())
        .flatten()
        .collect::<Vec<_>>();

    if files.is_empty() {
        return Err("At least one file must be specified. ".to_string());
    }

    for p in files {
        process(&p, args.fullname).map_err(|e| e.to_string())?;
    }

    println!("Done!");

    Ok(())
}

fn process<P: AsRef<Path>>(path: P, fullname: bool) -> Result<(), Box<dyn Error>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"【.+?】|\(.+? Edition\)").unwrap();
    }
    let path = path.as_ref();

    let destdirname = match path.file_stem().and_then(|n| n.to_str()) {
        Some(f) => f,
        None => return Ok(()),
    };
    let destdirname2 = if fullname {
        Cow::from(destdirname)
    } else {
        RE.replace_all(destdirname, "")
    };

    print!("Processing {} ...", &destdirname2);

    let destdir = Path::new(destdirname2.as_ref().trim());
    create_dir(destdir).ok();

    let mut counter = 0usize;

    let zip = File::open(path)?;
    let mut archive = ZipArchive::new(zip)?;
    for filename in archive
        .file_names()
        .filter(|f| f.ends_with(".png") || f.ends_with(".jpg"))
        .map(|f| f.into())
        .collect::<Vec<String>>()
    {
        let name = match Path::new(&filename).file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        let mut src = archive.by_name(&filename)?;
        if !src.is_file() {
            continue;
        }

        let mut dest = File::create(destdir.join(name))?;
        copy(&mut src, &mut dest)?;

        counter += 1;
    }

    println!(" {} images", counter);

    Ok(())
}
