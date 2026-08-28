use std::fmt;

use crate::{
    ErrorHandler,
    source::{ChData, Source},
    span::{RawSpan, Span},
};

pub struct Diagnostic {
    pub message: Message,
    pub help: Vec<Message>,
}

impl Diagnostic {
    pub(crate) fn new(message: String, span: Option<Span>) -> Self {
        Self {
            message: Message {
                value: message,
                span: span.into(),
            },
            help: Vec::new(),
        }
    }

    pub fn help_generic(&mut self, message: impl ToString) -> &mut Self {
        self.help.push(Message {
            value: message.to_string(),
            span: None,
        });
        self
    }
    pub fn help(&mut self, message: impl ToString, span: impl Into<Span>) -> &mut Self {
        self.help.push(Message {
            value: message.to_string(),
            span: Some(span.into()),
        });
        self
    }

    #[cfg(feature = "ascii-output")]
    pub fn write_to(
        &self,
        output: &mut impl fmt::Write,
        eh: &ErrorHandler,
        tag: &str,
        color: &str,
        help_tag: &str,
        help_color: &str,
    ) -> fmt::Result {
        let padding = self.message.write_to(output, eh, tag, color, None)?;
        let padding = padding.as_ref().map_or("", |s| s);

        for msg in &self.help {
            msg.write_to(output, eh, help_tag, help_color, Some(padding))?;
        }
        Ok(())
    }
}

pub struct Message {
    pub value: String,
    pub span: Option<Span>,
}

impl Message {
    #[cfg(feature = "ascii-output")]
    pub fn write_to(
        &self,
        output: &mut impl fmt::Write,
        eh: &ErrorHandler,
        tag: &str,
        fmt_hl: &str,
        original_padding: Option<&str>,
    ) -> Result<Option<String>, fmt::Error> {
        const FMT_RESET: &str = "\x1b[0m";
        const FMT_GUTTER: &str = "\x1b[97m";
        const FMT_ITALICS: &str = "\x1b[3m";
        const FMT_DIM: &str = "\x1b[2m";

        fn digits(x: usize) -> usize {
            x.ilog10() as usize + 1
        }

        let original_padding = original_padding.map_or(String::new(), |p| format!("{p}    "));

        // Print tag (error:, warn:, etc.)
        let message_prefix = if original_padding.is_empty() {
            ("", "")
        } else {
            (
                &original_padding[..original_padding.len() - 3],
                "\x1b[97m = ",
            )
        };
        writeln!(
            output,
            "{}{}{tag}:{FMT_RESET} {}",
            message_prefix.0, message_prefix.1, self.value,
        )?;

        let Some((
            SourceFragment {
                ln_start,
                col_start,
                ln_end,
                before,
                highlight,
                after,
            },
            source,
        )) = self.get_source_fragment(eh)
        else {
            return Ok(None);
        };
        let ln_digits = digits(ln_end);
        let padding = " ".repeat(ln_digits);

        // Print file source.
        writeln!(
            output,
            "{original_padding} {}{FMT_GUTTER}--> {FMT_RESET}{FMT_ITALICS}{}:{ln_start}:{col_start}{FMT_RESET}",
            padding, source.label
        )?;
        // Add an extra line as padding. Note that this doesn't have a reset code at the end.
        writeln!(output, "{original_padding}{FMT_GUTTER} {padding} |")?;

        // Line before highlight
        write!(
            output,
            "{original_padding}{FMT_GUTTER} {}{ln_start} | {FMT_RESET}{FMT_DIM}{before}{FMT_RESET}",
            &padding[..ln_digits - digits(ln_start)],
        )?;

        // Highlight
        for (i, line) in highlight.lines().enumerate() {
            if i != 0 {
                let ln = ln_start + i;
                write!(
                    output,
                    "\n{original_padding}{FMT_GUTTER} {}{ln} | {FMT_RESET}",
                    &padding[..ln_digits - digits(ln)]
                )?;
            }
            write!(output, "{fmt_hl}{line}{FMT_RESET}")?;
        }

        // Line after highlight
        writeln!(output, "{FMT_DIM}{after}{FMT_RESET}")?;

        // Add an extra line as padding.
        writeln!(output, "{original_padding}{FMT_GUTTER} {padding} |")?;
        Ok(Some(padding))
    }

    pub fn get_source_fragment<'a, 'b>(
        &self,
        eh: &'b ErrorHandler<'a>,
    ) -> Option<(SourceFragment<'a>, &'b Source<'a>)> {
        let (span, source) = eh.get_span_source(self.span?)?;
        // Clamp the span inside the source code.
        let span = RawSpan {
            start: span.start.min(source.code.len()),
            // Allow a span including an extra character past the end of the file.
            // This is to allow a `Span::char` call at the end of a file.
            end: span.end.min(source.code.len() + 1),
        };

        let ChData {
            ln: ln_start,
            col: col_start,
            line_span: first_line,
        } = source.get_ch_data(span.start);
        let ChData {
            ln: ln_end,
            col: _,
            line_span: _,
        } = source.get_ch_data(span.end);

        // If either the span extends beyond the source code
        if (span.end > source.code.len()
            // And it only selects one line
            && (ln_start + 1 == ln_end || ln_start == ln_end))
            // Or the span selects exclusively a newline character
            || &source.code[span] == "\n"
        {
            // Then highlight the end of the line.
            Some((
                SourceFragment {
                    ln_start,
                    col_start,
                    ln_end,
                    before: &source.code[first_line],
                    highlight: " ",
                    after: "",
                },
                source,
            ))
        } else {
            Some((
                SourceFragment {
                    ln_start,
                    col_start,
                    ln_end,
                    before: &source.code[first_line.start..span.start],
                    highlight: &source.code[span],
                    after: &source.code[span.end.min(source.code.len())..first_line.end],
                },
                source,
            ))
        }
    }
}

pub struct SourceFragment<'a> {
    ln_start: usize,
    col_start: usize,
    ln_end: usize,
    before: &'a str,
    highlight: &'a str,
    after: &'a str,
}
