use crate::renderer::{RenderOptions, Renderer};
use std::fmt::{Result, Write};

pub struct MarkdownRenderer<'a> {
    document: String,
    options: &'a RenderOptions,
}

impl<'a> Renderer<'a, String> for MarkdownRenderer<'a> {
    fn new(document: String, options: &'a RenderOptions) -> Self {
        Self { document, options }
    }

    fn options(&self) -> &'a RenderOptions {
        self.options
    }

    fn finish(self) -> String {
        self.document
    }

    fn render_heading(&mut self, text: &str, depth: usize) -> Result {
        // always ensure headings have empty line before them
        let second_last_char = self.document.chars().nth_back(2);
        if !matches!(second_last_char, Some('\n')) {
            writeln!(self.document)?;
        }

        writeln!(self.document, "{} {}\n", "#".repeat(depth + 1), text)
    }

    fn render_description(&mut self, text: &str, depth: usize) -> Result {
        for line in text.lines() {
            if let Some(stripped) = line.strip_prefix("# ") {
                self.render_heading(stripped, depth + 1)?;
            } else {
                writeln!(self.document, "{}", line)?;
            };
        }

        Ok(())
    }

    fn render_type(&mut self, text: &str) -> Result {
        writeln!(self.document, "> Type: `{}`\n", text)
    }

    fn render_text(&mut self, text: &str) -> Result {
        write!(self.document, "{}", text)
    }
}
