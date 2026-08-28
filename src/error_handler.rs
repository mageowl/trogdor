use std::{
    cell::OnceCell,
    fmt::{self, Display},
};

use crate::{
    diagnostic::Diagnostic,
    source::Source,
    span::{RawSpan, Span},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SourceId {
    pub(crate) index: usize,
    pub(crate) span_offset: u32,
}

impl SourceId {
    pub fn to_numerical_id(self) -> usize {
        self.index
    }
}

/// The error handler usually lasts for the entire lifetime of a compiler,
/// collecting errors and warnings along the way. You can use the [`is_ok`]
/// method to determine whether or not to stop a program, and the [`Display`]
/// implementation to provide a human-readable output. If you are in an
/// environment.
pub struct ErrorHandler<'a> {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    total_len: u32,
    pub(crate) sources: Vec<Source<'a>>,
}

impl<'a> ErrorHandler<'a> {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            total_len: 0,
            sources: Vec::new(),
        }
    }

    /// Add a source to the error handler, returning a [`SourceId`]. This ID
    /// can then be used to create [`Span`]s that let the user know where a
    /// diagnostic came from.
    pub fn add_source(&mut self, label: &'a str, code: &'a str) -> SourceId {
        const ERR_MESSAGE: &str = "sources larger than 4GiB in combined size will not be able to product coherent error sources.";
        let len: u32 = match code.len().try_into() {
            Ok(v) => v,
            Err(_) => {
                self.warn_generic(ERR_MESSAGE);
                0
            }
        };

        let span_offset = self.total_len;
        self.sources.push(Source {
            code: code,
            span_offset: span_offset,
            label: label,
            lines: OnceCell::new(),
        });
        self.total_len = match self.total_len.checked_add(len) {
            Some(v) => v,
            None => {
                self.warn_generic(ERR_MESSAGE);
                u32::MAX
            }
        };
        SourceId {
            index: self.sources.len() - 1,
            span_offset,
        }
    }

    /// Get the original source code from a [`SourceId`].
    pub fn get_source(&self, id: SourceId) -> &Source<'a> {
        &self.sources[id.index]
    }

    pub(crate) fn get_span_source(&self, mut span: Span) -> Option<(RawSpan, &Source<'a>)> {
        let mut sources = self.sources.iter();
        let mut prev = sources.next();
        for source in sources {
            if span.start < source.span_offset {
                span.start -= source.span_offset;
                span.end -= source.span_offset;
                break;
            } else {
                prev = Some(source);
            }
        }
        prev.map(|src| {
            (
                RawSpan {
                    start: span.start as usize,
                    end: span.end as usize,
                },
                src,
            )
        })
    }
    pub fn get_span_source_id(&self, span: Span) -> Option<SourceId> {
        let mut sources = self.sources.iter().enumerate();
        let mut prev = sources.next().map(|(i, source)| SourceId {
            index: i,
            span_offset: source.span_offset,
        });
        for (i, source) in sources {
            if span.start < source.span_offset {
                break;
            } else {
                prev = Some(SourceId {
                    index: i,
                    span_offset: source.span_offset,
                });
            }
        }
        prev
    }

    pub fn err(&mut self, message: impl ToString, span: impl Into<Span>) -> &mut Diagnostic {
        self.errors
            .push(Diagnostic::new(message.to_string(), Some(span.into())));
        self.errors.last_mut().unwrap()
    }
    pub fn warn(&mut self, message: impl ToString, span: Span) -> &mut Diagnostic {
        self.warnings
            .push(Diagnostic::new(message.to_string(), Some(span)));
        self.warnings.last_mut().unwrap()
    }

    pub fn err_generic(&mut self, message: impl ToString) -> &mut Diagnostic {
        self.errors.push(Diagnostic::new(message.to_string(), None));
        self.errors.last_mut().unwrap()
    }
    pub fn warn_generic(&mut self, message: impl ToString) -> &mut Diagnostic {
        self.warnings
            .push(Diagnostic::new(message.to_string(), None));
        self.warnings.last_mut().unwrap()
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

#[cfg(feature = "ascii-output")]
impl Display for ErrorHandler<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const HELP_TAG: &str = "\x1b[94mhelp";
        const HELP_HL: &str = "\x1b[4:5;58:5:12m";

        for (i, warn) in self.warnings.iter().enumerate() {
            warn.write_to(
                f,
                &self,
                "\x1b[93mwarning",
                "\x1b[4:3;58:5:11m",
                HELP_TAG,
                HELP_HL,
            )?;
            if i < self.warnings.len() - 1 || !self.errors.is_empty() {
                f.write_str("\n")?;
            }
        }
        for (i, error) in self.errors.iter().enumerate() {
            error.write_to(
                f,
                &self,
                "\x1b[31merror",
                "\x1b[4:3;58:5:1m",
                HELP_TAG,
                HELP_HL,
            )?;
            if i < self.errors.len() - 1 {
                f.write_str("\n")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{ErrorHandler, Span, error_handler::SourceId};
    use docstr::docstr;
    use expect_test::{Expect, expect};
    use regex::Regex;

    #[test]
    fn source_offsets() {
        let mut eh = ErrorHandler::new();
        let source1 = eh.add_source("<source1>", "abc");
        let source2 = eh.add_source("<source2>", "def");
        assert_eq!(eh.total_len, 6);
        let span1 = Span::new(0..3, source1);
        assert_eq!(span1.start, 0);
        assert_eq!(span1.end, 3);
        let span2 = Span::new(0..3, source2);
        assert_eq!(span2.start, 3);
        assert_eq!(span2.end, 6);
    }

    fn test_output(
        source: &str,
        expected: Expect,
        setup: impl FnOnce(&mut ErrorHandler, SourceId),
    ) {
        let mut eh = ErrorHandler::new();
        let source = eh.add_source("<test>", source);
        setup(&mut eh, source);

        let re = Regex::new(r"\x1b\[[^m]+m").unwrap();
        let output = eh.to_string();

        println!("Displayed:");
        println!("{output}");

        let stripped = re.replace_all(&output, "");
        expected.assert_eq(&stripped);
    }

    const SOURCE: &str = docstr!(
        /// let random_number = 185
        /// print(random_number)
    );

    #[test]
    fn write_diagnostics() {
        test_output(SOURCE, EXPECTED, |eh, source| {
            eh.err(
                "expected a statement, instead found 'let'",
                Span::new(0..3, source),
            );
        });
        const EXPECTED: Expect = expect![["
            error: expected a statement, instead found 'let'
              --> <test>:1:1
               |
             1 | let random_number = 185
               |
        "]];
    }

    #[test]
    fn help_messages() {
        test_output(SOURCE, EXPECTED, |eh, source| {
            let span = Span::new(0..3, source);
            eh.warn("i dont like let keywords", span)
                .help_generic("try a different keyword")
                .help("found here", span);
        });
        const EXPECTED: Expect = expect![["
            warning: i dont like let keywords
              --> <test>:1:1
               |
             1 | let random_number = 185
               |
               = help: try a different keyword
               = help: found here
                   --> <test>:1:1
                    |
                  1 | let random_number = 185
                    |
        "]];
    }

    #[test]
    fn end_of_file() {
        test_output(SOURCE, EXPECTED, |eh, source| {
            eh.err("unexpected end of file", Span::new(43..44, source));
        });
        const EXPECTED: Expect = expect![["
            error: unexpected end of file
              --> <test>:2:20
               |
             2 | print(random_number) 
               |
        "]];
    }
}
