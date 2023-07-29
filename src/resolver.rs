use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use syn::Item;

use crate::module_path::ModulePath;
use crate::parser::extract_doc_comment;
use crate::utils::PathExt;
use crate::{parser, ElementInfo, FileInfo, Info};

pub type ModuleCache = HashMap<ModulePath, FileInfo>;

pub struct Resolver {
    /// Path to the entry file (`main.rs` or `lib.rs`)
    entry_file: String,
    /// Path to the project `src` folder
    entry_path: PathBuf,
    /// All resolved modules
    module_cache: ModuleCache,
}

impl Resolver {
    pub fn new<P: AsRef<Path>>(entry_file: P) -> Self {
        let entry_path = entry_file
            .as_ref()
            .parent()
            .expect("parent path to exist")
            .to_path_buf();

        let entry_file = entry_file
            .as_ref()
            .file_name()
            .expect("file to exist")
            .to_str()
            .expect("to be valid string")
            .to_string();

        Self {
            entry_file,
            entry_path,
            module_cache: HashMap::new(),
        }
    }

    pub fn resolve(&mut self) -> Result<()> {
        self.resolve_module(ModulePath::new())
    }

    fn resolve_module(&mut self, module_path: ModulePath) -> Result<()> {
        let path = module_path.as_path(&self.entry_path, &self.entry_file);

        let file_name = path
            .file_stem()
            .expect("file name to exist")
            .to_str()
            .expect("file name to be valid utf-8")
            .to_string();

        let file = read_file(&path)?;
        let items = Self::get_module_items(file.items, &module_path)?;

        let info = {
            FileInfo {
                // private fields kept in case they're needed
                _name: file_name,
                _path: path,
                elements: items.elements,
            }
        };

        self.module_cache.insert(module_path, info);

        for import in items.modules {
            if !self.module_cache.contains_key(&import) {
                self.resolve_module(import)?;
            }
        }

        Ok(())
    }

    fn get_module_items(items: Vec<Item>, module_path: &ModulePath) -> Result<ModuleItems> {
        let mut modules = vec![];
        let mut elements = vec![];

        for item in items {
            match item {
                Item::Mod(module) => {
                    if let Some((_, items)) = module.content {
                        Self::get_module_items(items, module_path)?;
                    } else {
                        modules.push(module_path.join(module.ident.to_string()));
                    }
                }
                Item::Enum(item_enum) => elements.push(Info {
                    name: item_enum.ident.to_string(),
                    description: extract_doc_comment(&item_enum.attrs),
                    element: ElementInfo::Enum(parser::parse_enum(item_enum)),
                }),
                Item::Struct(item_struct) => elements.push(Info {
                    name: item_struct.ident.to_string(),
                    description: extract_doc_comment(&item_struct.attrs),
                    element: ElementInfo::Struct(parser::parse_struct(item_struct)),
                }),
                _ => {}
            }
        }

        Ok(ModuleItems { modules, elements })
    }

    pub fn resolve_absolute(&self, path: &ModulePath) -> Option<&Info> {
        let parent = path.parent();
        let element = path.element();

        if let Some(element) = element {
            self.module_cache
                .get(&parent)
                .and_then(|file| file.elements.iter().find(|el| el.name == element))
        } else {
            None
        }
    }

    pub fn resolve_shorthand(&self, element: &str) -> Option<&Info> {
        let results = self
            .module_cache
            .iter()
            .filter_map(|(path, _)| self.resolve_absolute(&path.join(element)))
            .collect::<Vec<_>>();

        if results.len() == 1 {
            results.first().copied()
        } else {
            None
        }
    }
}

/// Attempts to find the entrypoint to the project, relative to the given path.
pub fn find_entry_file<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let path = path
        .as_ref()
        .join_if_exists("src")
        .join_if_exists("main.rs")
        .join_if_exists("lib.rs");

    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn read_file<P: AsRef<Path>>(module: P) -> Result<syn::File> {
    let str = fs::read_to_string(module)?;
    let tree = syn::parse_file(&str)?;

    Ok(tree)
}

struct ModuleItems {
    modules: Vec<ModulePath>,
    elements: Vec<Info>,
}
