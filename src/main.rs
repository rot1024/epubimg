use clap::Clap;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    borrow::Cow,
    error::Error,
    fs::{create_dir, File},
    io::copy,
    path::Path,
    process::exit,
};
use zip::ZipArchive;

#[derive(Debug, Clap)]
#[clap(about = "Image file extractor from ePub file")]
struct App {
    #[clap(about = "epub or zip file paths")]
    files: Vec<String>,
    #[clap(short, long, about = "Use full name for output directory name")]
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

    if args.files.is_empty() {
        return Err("At least one file must be specified. ".to_string());
    }

    for p in args.files {
        process(&p, args.fullname).map_err(|e| e.to_string())?;
    }

    println!("Done!");

    Ok(())
}

fn process(p: &str, fullname: bool) -> Result<(), Box<dyn Error>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"【.+?】|\(.+? Edition\)").unwrap();
    }

    let path = Path::new(&p);
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
