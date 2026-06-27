use std::{
    char,
    fmt::{self, Display},
    ops::Range,
};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
}

#[derive(Debug, Clone, Copy)]
enum GraphemeWidth {
    Half,
    Full,
}

impl GraphemeWidth {
    const fn saturating_add(self, other: usize) -> usize {
        return match self {
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        };
    }
}

#[derive(Debug)]
struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        let fragments = Self::text_to_fragment(line_str);
        return Self { fragments };
    }

    fn text_to_fragment(line_str: &str) -> Vec<TextFragment> {
        return line_str
            .graphemes(true)
            .map(Self::grapheme_to_fragment)
            .collect();
    }

    fn grapheme_to_fragment(g: &str) -> TextFragment {
        debug_assert_eq!(g.graphemes(true).count(), 1);

        let width = g.width();
        let rendered_width = match width {
            0 | 1 => GraphemeWidth::Half,
            _ => GraphemeWidth::Full,
        };

        let mut replacement = match width {
            0 => Some('.'),
            _ => None,
        };

        if g == "\t" {
            replacement = Some(' ');
        } else if g.trim().is_empty() && g != " " {
            replacement = Some('␣');
        } else {
            g.chars().for_each(|c| {
                if c.is_control() {
                    replacement = Some('▯');
                }
            });
        }

        return TextFragment {
            grapheme: g.into(),
            rendered_width,
            replacement,
        };
    }

    pub fn get_visible_graphemes(&self, range: Range<usize>) -> String {
        let mut result = String::new();
        if range.start >= range.end {
            return result;
        }

        let mut current_pos = 0;
        for f in &self.fragments {
            let fragment_end = f.rendered_width.saturating_add(current_pos);
            if current_pos >= range.end {
                break;
            }

            if fragment_end > range.start {
                match (
                    fragment_end > range.end || current_pos < range.start,
                    f.replacement,
                ) {
                    (true, _) => result.push('⋯'),
                    (false, Some(c)) => result.push(c),
                    (false, None) => {
                        result.push_str(&f.grapheme);
                    }
                }
            }
            current_pos = fragment_end;
        }
        return result;
    }

    pub fn insert_char(&mut self, ch: char, target_idx: usize) {
        #![allow(clippy::implicit_return)] // implicit return do actually look better in closures
        let res: String = self
            .fragments
            .iter()
            .enumerate()
            .flat_map(|(idx, frag)| {
                let insert = (idx == target_idx).then_some(ch);
                insert.into_iter().chain(frag.grapheme.chars())
            })
            .chain((target_idx >= self.fragments.len()).then_some(ch))
            .collect();

        self.fragments = Self::text_to_fragment(&res);
    }

    pub fn delete_char(&mut self, target_idx: usize) {
        #![allow(clippy::implicit_return)] // implicit return do actually look better in closures
        let result: String = self
            .fragments
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx != target_idx)
            .map(|(_, f)| f.grapheme.as_str())
            .collect();

        self.fragments = Self::text_to_fragment(&result);
    }

    pub fn grapheme_count(&self) -> usize {
        return self.fragments.len();
    }

    pub fn width_until(&self, grapheme_idx: usize) -> usize {
        return self
            .fragments
            .iter()
            .take(grapheme_idx)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => return 1,
                GraphemeWidth::Full => return 2,
            })
            .sum();
    }

    pub fn append_line(&mut self, next_line: &Line) {
        #![allow(clippy::implicit_return)]
        let mut result: String = self.to_string();
        let next_string: String = next_line.to_string();
        result.push_str(&next_string);
        self.fragments = Self::text_to_fragment(&result);
    }

    pub fn split(&mut self, grapheme_idx: usize) -> Self {
        if grapheme_idx > self.fragments.len() {
            return Self::default();
        }
        let fragments = self.fragments.split_off(grapheme_idx);
        return Line { fragments };
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #![allow(clippy::implicit_return)]
        let result: String = self.fragments.iter().map(|f| f.grapheme.as_str()).collect();

        return f.write_str(&result);
    }
}
