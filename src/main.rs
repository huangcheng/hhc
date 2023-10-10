use std::env;
use std::fs;
use std::path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use clap::Parser;

use libhhc::core::Doc;
use libhhc::core::{gpx, kml, tcx};

mod cli;

use cli::Args;

const FILE_EXTENSIONS: [&str; 3] = ["gpx", "kml", "tcx"];
const FILE_PREFIX: &str = "_converted";

struct File {
    path: String,
    content: String,
}

fn main() {
    let args = Args::parse();

    let cwd = if let Some(path) = args.path {
        path
    } else {
        env::current_dir().unwrap().to_str().unwrap().to_string()
    };

    let overwrite = args.overwrite;

    let mut handles: Vec<JoinHandle<std::io::Result<()>>> = vec![];

    let (tx_files, rx_files): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (tx_contents, rx_contents): (Sender<File>, Receiver<File>) = mpsc::channel();

    handles.push(thread::spawn(move || {
        let mut files: Vec<String> = vec![];

        for entry in fs::read_dir(cwd).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_file() {
                if let Some(file_extension) = path.extension() {
                    if FILE_EXTENSIONS.contains(&file_extension.to_str().unwrap()) {
                        files.push(path.to_str().unwrap().to_string());
                    }
                }
            }
        }

        for file in files {
            if tx_files.send(file).is_err() {
                continue;
            }
        }

        Ok(())
    }));

    handles.push(thread::spawn(move || {
        for file in rx_files {
            let file_content = fs::read_to_string(file.clone()).unwrap();

            let file = File {
                path: file,
                content: file_content,
            };

            if tx_contents.send(file).is_err() {
                continue;
            }
        }

        Ok(())
    }));

    handles.push(thread::spawn(move || {
        for file in rx_contents {
            println!("converting {}", file.path);

            let file_path = file.path;
            let file_content = file.content;

            let file_name = path::Path::new(&file_path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let file_extension = path::Path::new(&file_path)
                .extension()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            let file_name = file_name.replace(&format!(".{}", file_extension), "");

            let content = match file_extension.as_str() {
                "gpx" => gpx::Document::convert(&file_content).unwrap(),
                "kml" => kml::Document::convert(&file_content).unwrap(),
                "tcx" => tcx::Document::convert(&file_content).unwrap(),
                _ => panic!("Unsupported file extension: {}", file_extension),
            };

            let file_name = if overwrite {
                file_name
            } else {
                format!("{}{}", file_name, FILE_PREFIX)
            };

            let new_file_name = format!("{}.{}", file_name, file_extension);

            let new_file_path = path::Path::new(&file_path)
                .parent()
                .unwrap()
                .join(new_file_name);

            fs::write(new_file_path, content).unwrap();

            println!("{} converted", file_path);
        }

        Ok(())
    }));

    for handle in handles {
        handle.join().unwrap().unwrap();
    }
}
