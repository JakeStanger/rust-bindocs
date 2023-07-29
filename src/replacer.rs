use serde::Deserialize;
use std::fmt::Write;
use std::marker::PhantomData;
use tracing::error;

use crate::renderer::Renderer;
use crate::resolver::Resolver;

#[derive(Deserialize, Debug)]
pub struct ReplaceOptions {
    /// Whether to include the element header.
    /// Defaults to true.
    #[serde(default = "default_true")]
    pub header: bool,

    /// The current heading depth, starting at `0`.
    /// Headings will be placed at one more than the current depth.
    /// For example, if the next heading should be `## h2`, use a depth of `1`.
    #[serde(default = "default_depth")]
    pub depth: usize,
}

impl Default for ReplaceOptions {
    fn default() -> Self {
        Self {
            header: true,
            depth: 1,
        }
    }
}

impl ReplaceOptions {
    /// Attempts to parse the string using `Corn`.
    ///
    /// If invalid, the error is logged
    /// and the default options are returned instead.
    fn parse_or_default(str: &str) -> Self {
        libcorn::from_str::<ReplaceOptions>(str).unwrap_or_else(|err| {
            error!("Invalid replace options:\n{err}");
            ReplaceOptions::default()
        })
    }
}

const fn default_true() -> bool {
    true
}

const fn default_depth() -> usize {
    1
}

pub struct Replacer<'a, R, W>
where
    R: Renderer<'a, W>,
    W: Write,
{
    renderer: R,
    resolver: &'a Resolver,
    _phantom: PhantomData<W>,
}

impl<'a, R, W> Replacer<'a, R, W>
where
    R: Renderer<'a, W>,
    W: Write,
{
    pub fn new(renderer: R, resolver: &'a Resolver) -> Self {
        Self {
            renderer,
            resolver,
            _phantom: PhantomData,
        }
    }

    pub fn replace(&mut self, input: String) {
        let mut chars = input.chars().collect::<Vec<_>>();

        while !chars.is_empty() {
            let char_pair = if chars.len() > 1 {
                Some(&chars[..=1])
            } else {
                None
            };

            let skip = match char_pair {
                Some(['<', '%']) => self.parse_token(&chars),
                _ => self.parse_static(&chars),
            };

            // quick runtime check to make sure the parser is working as expected
            assert_ne!(skip, 0);

            chars.drain(..skip);
        }
    }

    fn parse_token(&mut self, chars: &[char]) -> usize {
        const SKIP_CHARS: usize = 4; // two control characters

        let str = chars
            .windows(2)
            .skip(2)
            .take_while(|win| win != &['%', '>'])
            .map(|w| w[0])
            .collect::<String>();

        let trimmed = str.trim();
        let (path, opts) = trimmed
            .split_once(' ')
            .map(|(path, opts)| (path, ReplaceOptions::parse_or_default(opts)))
            .unwrap_or_else(|| (trimmed, ReplaceOptions::default()));

        let info = self
            .resolver
            .resolve_absolute(&path.into())
            .or_else(|| self.resolver.resolve_shorthand(path));

        if let Some(info) = info {
            self.renderer.render_element(info, opts).unwrap();
        } else {
            self.renderer.render_text(&format!("<% {str} %>")).unwrap();
        };

        str.chars().count() + SKIP_CHARS
    }

    fn parse_static(&mut self, chars: &[char]) -> usize {
        let mut str = chars
            .windows(2)
            .take_while(|&win| win != ['<', '%'])
            .map(|w| w[0])
            .collect::<String>();

        // if segment is at end of string, last char gets missed above due to uneven window.
        if chars.len() == str.len() + 1 {
            let remaining_char = *chars.get(str.len()).expect("Failed to find last char");
            str.push(remaining_char);
        }

        self.renderer.render_text(&str).unwrap();

        str.chars().count()
    }

    pub(crate) fn finish(self) -> W {
        self.renderer.finish()
    }
}
