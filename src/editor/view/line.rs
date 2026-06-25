use std::{char, ops::Range};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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
        let fragments: Vec<TextFragment> = line_str
            .graphemes(true)
            .map(|g| {
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
                    replacement = Some('t');
                }

                return TextFragment {
                    grapheme: g.into(),
                    rendered_width,
                    replacement,
                };
            })
            .collect();

        return Self { fragments };
    }

    pub fn get_visible_graphemes(&self, range: Range<usize>) -> String {
        let mut result = String::new();
        if range.start >= range.end {
            return result;
        }

        let mut current_pos = 0;
        for f in &self.fragments {
            let fragment_end = f.rendered_width.saturating_add(current_pos);
            eprintln!(
                "grapheme: {:?}, replacement: {:?}, current_pos: {}, fragment_end: {}, range: {:?}",
                f.grapheme, f.replacement, current_pos, fragment_end, range
            );

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

    pub fn grapheme_len(&self) -> usize {
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
}
