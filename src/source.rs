use std::cell::OnceCell;

use crate::span::RawSpan;

pub struct Source<'a> {
    pub(crate) span_offset: u32,
    pub(crate) code: &'a str,
    pub(crate) label: &'a str,
    pub(crate) lines: OnceCell<Vec<RawSpan>>,
}

impl<'a> Source<'a> {
    pub(crate) fn lines(&self) -> &Vec<RawSpan> {
        self.lines.get_or_init(|| Self::gen_lines(self.code))
    }

    fn gen_lines(code: &'a str) -> Vec<RawSpan> {
        let mut lines = Vec::new();
        let mut ch = 0;
        for line in code.lines() {
            lines.push(RawSpan::from(ch..ch + line.len()));
            // +1 for newline character
            ch += line.len() + 1;
        }

        lines
    }

    pub(crate) fn get_ch_data(&self, ch: usize) -> ChData {
        let lines = self.lines();
        for (ln, span @ RawSpan { start, end }) in lines.iter().enumerate() {
            if ch <= *end {
                return ChData {
                    ln: ln + 1,
                    col: (ch - start + 1) as usize,
                    line_span: *span,
                };
            }
        }
        return lines.last().map_or(
            ChData {
                ln: lines.len() + 1,
                col: 1,
                line_span: RawSpan::char(self.code.len() - 1),
            },
            |span| ChData {
                ln: lines.len(),
                col: (ch - span.start + 1) as usize,
                line_span: *span,
            },
        );
    }

    pub fn code(&self) -> &'a str {
        self.code
    }
    pub fn label(&self) -> &'a str {
        self.label
    }
    pub fn span_offset(&self) -> u32 {
        self.span_offset
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct ChData {
    pub(crate) ln: usize,
    pub(crate) col: usize,
    pub(crate) line_span: RawSpan,
}

#[cfg(test)]
mod test {
    use std::cell::OnceCell;

    use crate::{
        source::{ChData, Source},
        span::RawSpan,
    };

    #[test]
    fn ch_data() {
        let source = Source {
            code: "line 1\nline 2\nline 3",
            span_offset: 1,
            label: "<test>",
            lines: OnceCell::new(),
        };
        assert_eq!(
            source.get_ch_data(3),
            ChData {
                ln: 1,
                col: 4,
                line_span: RawSpan::from(0..6)
            }
        );
        assert_eq!(
            source.get_ch_data(12),
            ChData {
                ln: 2,
                col: 6,
                line_span: RawSpan::from(7..13)
            }
        );
        assert_eq!(
            source.get_ch_data(15),
            ChData {
                ln: 3,
                col: 2,
                line_span: RawSpan::from(14..20)
            }
        );
    }
}
