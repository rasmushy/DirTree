use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process;

const EXCLUDE_DIRS: &[&str] = &[
    "target",
    "node_modules",
    ".git",
    "dist",
    "build",
    "out",
    "pkg",
];

#[derive(Parser, Debug)]
#[command(name = "dirtree", about = "Display the directory structure.", long_about = None)]
struct Args {
    /// The root path to display the tree for.
    #[arg(default_value = ".")]
    path: String,
    /// Directories to exclude from the tree.
    #[clap(short, long)]
    exclude: Vec<String>,
}


fn create_indentation(depth: usize, path_last: &[bool]) -> String {
    let mut indentation = String::new();
    for &last in path_last.iter().take(depth - 1) {
        indentation.push_str(if last { "    " } else { "│   " });
    }
    if depth > 0 {
        indentation.push_str(if *path_last.last().unwrap_or(&false) {
            "└── "
        } else {
            "├── "
        });
    }
    indentation
}

fn print_indented(text: &str, is_dir: bool, indentation: &str) {
    if is_dir {
        println!("{}{}/", indentation, text);
    } else {
        println!("{}{}", indentation, text);
    }
}

fn should_exclude(entry_name: &str, exclude_dirs: &[String]) -> bool {
    exclude_dirs.iter().any(|dir| dir == entry_name)
}

fn tree(
    current_path: PathBuf,
    depth: usize,
    path_last: &mut Vec<bool>,
    exclude_dirs: &[String],
) -> Result<(), std::io::Error> {
    let mut entries = fs::read_dir(&current_path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            let name = path.file_name()?.to_string_lossy().into_owned();
            if should_exclude(&name, exclude_dirs) {
                None
            } else {
                Some((path, name))
            }
        })
        .collect::<Vec<(PathBuf, String)>>();

    entries.sort_by(|a, b| a.1.cmp(&b.1));

    for (i, (path, name)) in entries.iter().enumerate() {
        let is_dir = path.is_dir();
        let new_is_last = i == entries.len() - 1;

        path_last.push(new_is_last);
        print_indented(&name, is_dir, &create_indentation(depth, path_last));

        if is_dir {
            tree(path.to_path_buf(), depth + 1, path_last, exclude_dirs)?;
        }

        path_last.pop();
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    let path = PathBuf::from(&args.path);

    let exclude_dirs = if args.exclude.is_empty() {
        EXCLUDE_DIRS.iter().map(|s| s.to_string()).collect()
    } else {
        args.exclude
    };

    let display_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    println!("{}/", display_name);
    let mut path_last = Vec::new();
    if let Err(e) = tree(path, 1, &mut path_last, &exclude_dirs) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
