use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const INCLUDED_EXTENSIONS: &[&str] = &["rs", "toml", "lock", "wgsl", "md", "json"];
const EXCLUDED_DIRS: &[&str] = &[
    "target", ".git", ".idea", ".vscode", "node_modules", "coalesced",
];
const EXCLUDED_FILES: &[&str] = &[".env", ".env.local", ".env.production", "secrets.json"];

fn main() {
    if let Err(error) = run() {
        eprintln!("Coalesce failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let Some(folder) = rfd::FileDialog::new().pick_folder() else {
        println!("No folder selected.");
        return Ok(());
    };

    let project_name = folder
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid project folder name"))?;

    // Relative to Coalesce's executable, rather than the shell's working directory.
    let output_dir = PathBuf::from("./coalesced").join(project_name);
    fs::create_dir_all(&output_dir)?;
    let output_file = output_dir.join(format!("{project_name}.md"));

    let mut files = Vec::new();
    let mut tree = String::new();
    let root_label = format!("{project_name}/");
    tree.push_str(&root_label);
    tree.push('\n');
    collect_tree(&folder, "", &mut tree, &mut files)?;

    let mut markdown = String::new();
    markdown.push_str(&format!("# Project: {project_name}\n\n"));
    markdown.push_str(&format!("Included source files: {}\n\n", files.len()));
    markdown.push_str("## Directory Tree\n\n```text\n");
    markdown.push_str(&tree);
    markdown.push_str("```\n\n");

    let mut exported = 0usize;
    for path in &files {
        let relative = path.strip_prefix(&folder).map_err(io::Error::other)?;
        let display_path = relative.to_string_lossy().replace('\\', "/");
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) => {
                eprintln!("Skipping {}: {error}", path.display());
                continue;
            }
        };
        let language = match path.extension().and_then(OsStr::to_str).unwrap_or("") {
            "rs" => "rust",
            "toml" => "toml",
            "wgsl" => "wgsl",
            "json" => "json",
            "md" => "markdown",
            _ => "text",
        };
        // A longer fence prevents embedded Markdown code fences from ending the block.
        let fence = "`".repeat(longest_backtick_run(&contents).max(2) + 1);
        markdown.push_str(&format!("## File: {display_path}\n\n{fence}{language}\n"));
        markdown.push_str(&contents);
        if !contents.ends_with('\n') {
            markdown.push('\n');
        }
        markdown.push_str(&format!("{fence}\n\n"));
        exported += 1;
    }

    fs::write(&output_file, markdown)?;
    println!("Selected: {}", folder.display());
    println!("Project: {project_name}");
    println!("Exported {exported} files to {}", output_file.display());
    Ok(())
}

fn collect_tree(
    directory: &Path,
    prefix: &str,
    tree: &mut String,
    files: &mut Vec<PathBuf>,
) -> io::Result<()> {
    let mut entries = fs::read_dir(directory)?
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut included = Vec::new();
    for entry in entries {
        let kind = entry.file_type()?;
        let name = entry.file_name();
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            if !EXCLUDED_DIRS.iter().any(|excluded| name == OsStr::new(excluded)) {
                included.push((entry, true));
            }
        } else if kind.is_file() {
            let path = entry.path();
            let allowed = path
                .extension()
                .and_then(OsStr::to_str)
                .is_some_and(|ext| INCLUDED_EXTENSIONS.contains(&ext));
            let secret = EXCLUDED_FILES.iter().any(|excluded| name == OsStr::new(excluded))
                || name.to_string_lossy().starts_with(".env.");
            if allowed && !secret {
                included.push((entry, false));
            }
        }
    }

    let count = included.len();
    for (index, (entry, is_dir)) in included.into_iter().enumerate() {
        let last = index + 1 == count;
        let name = entry.file_name().to_string_lossy().into_owned();
        tree.push_str(prefix);
        tree.push_str(if last { "└── " } else { "├── " });
        tree.push_str(&name);
        if is_dir {
            tree.push('/');
        }
        tree.push('\n');
        if is_dir {
            let next_prefix = format!("{prefix}{}", if last { "    " } else { "│   " });
            collect_tree(&entry.path(), &next_prefix, tree, files)?;
        } else {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn longest_backtick_run(text: &str) -> usize {
    let mut max_run = 0;
    let mut current = 0;
    for byte in text.bytes() {
        if byte == b'`' {
            current += 1;
            max_run = max_run.max(current);
        } else {
            current = 0;
        }
    }
    max_run
}
