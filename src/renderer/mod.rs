mod markdown;

use crate::replacer::ReplaceOptions;
use crate::{ElementInfo, EnumInfo, FieldInfo, Info, StructInfo};
pub use markdown::MarkdownRenderer;
use std::fmt::{Result, Write};

#[derive(Debug)]
pub struct RenderOptions {
    pub simplified_types: bool,
}

pub trait Renderer<'a, W: Write> {
    fn new(document: W, options: &'a RenderOptions) -> Self;
    fn options(&self) -> &'a RenderOptions;
    fn finish(self) -> W;

    fn render_heading(&mut self, text: &str, depth: usize) -> Result;
    fn render_description(&mut self, text: &str, depth: usize) -> Result;
    fn render_type(&mut self, text: &str) -> Result;
    fn render_text(&mut self, text: &str) -> Result;

    fn render_element(&mut self, info: &Info, options: ReplaceOptions) -> Result {
        let depth = options.depth;

        if options.header {
            self.render_heading(&info.name, depth)?;
        }

        self.render_description(&info.description, depth)?;

        match &info.element {
            ElementInfo::Struct(info) => self.render_struct(info, depth + 1),
            ElementInfo::Enum(info) => self.render_enum(info, depth + 1),
        }?;

        Ok(())
    }

    fn render_struct(&mut self, info: &StructInfo, depth: usize) -> Result {
        for field in &info.fields {
            self.render_field(field, depth)?;
        }

        Ok(())
    }

    fn render_enum(&mut self, info: &EnumInfo, depth: usize) -> Result {
        for variant in &info.variants {
            self.render_heading(&variant.name, depth)?;
            self.render_description(&variant.description, depth)?;

            for field in &variant.fields {
                self.render_field(field, depth + 1)?;
            }
        }

        Ok(())
    }

    fn render_field(&mut self, info: &FieldInfo, depth: usize) -> Result {
        self.render_heading(&info.name, depth)?;
        self.render_type(&info.ty.to_doc_string(self.options().simplified_types))?;
        self.render_description(&info.description, depth)?;

        Ok(())
    }
}
