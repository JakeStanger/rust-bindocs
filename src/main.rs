mod module_path;
mod parser;
mod renderer;
mod replacer;
mod resolver;
mod utils;

use clap::Parser;
use std::fmt::{Display, Formatter};
use std::fs;

use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Instant;

use crate::renderer::{MarkdownRenderer, RenderOptions, Renderer};
use crate::replacer::Replacer;
use crate::resolver::Resolver;
use color_eyre::Result;
use pathdiff::diff_paths;
use tracing::{error, info};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
struct Args {
    /// Path to the crate root.
    /// Defaults to current dir.
    #[arg(short, long, default_value = ".")]
    project_path: PathBuf,

    /// Path to the document templates(s).
    /// This can be a file name for a single file, or a directory for multiple.
    /// Defaults to `<project_path>/docs`.
    #[arg(short, long)]
    docs_path: Option<PathBuf>,

    /// Path to output the rendered doc(s).
    /// This can be a file name for a single file, or directory for multiple.
    /// Defaults to `<project_path>/target/bindoc`.
    #[arg(short, long)]
    output_path: Option<PathBuf>,
}

#[derive(Debug, Default)]
pub struct TypeInfo {
    name: String,
    generics: Vec<TypeInfo>,
}

#[derive(Debug)]
pub struct FieldInfo {
    name: String,
    description: String,
    ty: TypeInfo,
}

#[derive(Debug)]
pub struct StructInfo {
    fields: Vec<FieldInfo>,
}

#[derive(Debug)]
pub struct VariantInfo {
    name: String,
    description: String,
    fields: Vec<FieldInfo>,
}

#[derive(Debug)]
pub struct EnumInfo {
    variants: Vec<VariantInfo>,
}

#[derive(Debug)]
pub enum ElementInfo {
    Struct(StructInfo),
    Enum(EnumInfo),
}

#[derive(Debug)]
pub struct Info {
    name: String,
    description: String,
    element: ElementInfo,
}

#[derive(Debug)]
pub struct FileInfo {
    _name: String,
    _path: PathBuf,
    elements: Vec<Info>,
}

impl Display for Info {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "# {}\n\n{}\n\n", self.name, self.description)
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let start_time = Instant::now();

    let mut args = Args::parse();

    if !args.project_path.exists() {
        eprintln!("Project path does not exist");
        exit(1);
    }

    let Some(entry) = resolver::find_entry_file(&args.project_path) else {
        eprintln!("Could not find Rust project at path");
        exit(2);
    };

    let docs_path = args
        .docs_path
        .take()
        .unwrap_or_else(|| args.project_path.join("docs"));

    if !docs_path.exists() {
        eprintln!("Documentation path does not exist");
        exit(1);
    }

    let output_path = args
        .output_path
        .take()
        .unwrap_or_else(|| args.project_path.join("target/bindoc"));

    let mut resolver = Resolver::new(entry);
    resolver.resolve()?;

    let options = RenderOptions {
        simplified_types: true,
    };

    if docs_path.is_file() {
        process_file(
            &docs_path,
            docs_path.parent().expect("parent path to exist"),
            &output_path,
            &resolver,
            &options,
        )?
    } else {
        if !output_path.exists() {
            fs::create_dir_all(&output_path)?;
        }

        for entry in WalkDir::new(&docs_path) {
            match entry {
                Ok(entry) if entry.file_type().is_file() => {
                    process_file(entry.path(), &docs_path, &output_path, &resolver, &options)?
                }
                Ok(_) => {}
                Err(err) => {
                    error!("Error walking directory: {err}");
                }
            }
        }
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    info!("Done in {} seconds", elapsed);

    Ok(())
}

fn process_file(
    file_path: &Path,
    docs_path: &Path,
    output_path: &Path,
    resolver: &Resolver,
    options: &RenderOptions,
) -> Result<()> {
    let file_output_path = if output_path_is_file_like(output_path) {
        output_path.to_path_buf()
    } else {
        let relative_path = diff_paths(file_path, docs_path).expect("relative path to exist");
        output_path.join(relative_path)
    };

    info!("Rendering file: {}", file_output_path.display());

    let output = render_file(file_path, resolver, options)?;
    write_file(&file_output_path, output)?;

    Ok(())
}

fn render_file(path: &Path, resolver: &Resolver, options: &RenderOptions) -> Result<String> {
    let output = String::new();
    let renderer = MarkdownRenderer::new(output, options);

    let input = fs::read_to_string(path)?;
    let mut replacer = Replacer::new(renderer, resolver);

    replacer.replace(input);
    Ok(replacer.finish())
}

fn write_file(path: &Path, contents: String) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, contents)?;

    Ok(())
}

fn output_path_is_file_like(output_path: &Path) -> bool {
    output_path.extension().is_some()
}
