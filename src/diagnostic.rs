use std::io;

use crate::span::Span;

pub struct Diagnostic {
    pub message: Message,
    pub help: Vec<Message>,
}

impl Diagnostic {
    pub fn new(message: impl ToString, span: impl Into<Option<Span>>) -> Self {
        Self {
            message: Message {
                value: message.to_string(),
                span: span.into(),
            },
            help: Vec::new(),
        }
    }

    pub fn help(&mut self, message: impl ToString, span: impl Into<Option<Span>>) -> &mut Self {
        self.help.push(Message {
            value: message.to_string(),
            span: span.into(),
        });
        self
    }

    pub fn write_to(
        &self,
        output: &mut impl io::Write,
        source: &str,
        tag: &str,
        color: &str,
        help_tag: &str,
        help_color: &str,
    ) -> io::Result<()> {
        self.message.write_to(output, source, tag, color)?;
        for msg in &self.help {
            msg.write_to(output, source, help_tag, help_color)?;
        }
        Ok(())
    }
}

pub struct Message {
    pub value: String,
    pub span: Option<Span>,
}

impl Message {
    pub fn write_to(
        &self,
        output: &mut impl io::Write,
        source: &str,
        tag: &str,
        color: &str,
    ) -> io::Result<()> {
        println!("\n{color}{tag}\x1b[0;1m: {}\x1b[22m", self.value);
        if let Some(span) = self.span {
            let lines = source
                .lines()
                .skip(span.start.ln)
                .take(span.end.ln - span.start.ln + 1)
                .enumerate();
            let len = lines.size_hint().1.map(|l| l - 1);
            for (i, line) in lines {
                let first = i == 0;
                let last = Some(i) == len;
                if first && last {
                    write!(
                        output,
                        "\x1b[2m{} |\x1b[22m {}{color}{}\x1b[0m{}",
                        span.start.ln + i + 1,
                        &line[0..span.start.col].replace("\t", "    "),
                        &line[span.start.col..span.end.col].replace("\t", "    "),
                        &line[span.end.col..].replace("\t", "    ")
                    )?;
                } else if first {
                    write!(
                        output,
                        "\x1b[2m{} |\x1b[22m {}{color}{}\x1b[0m",
                        span.start.ln + i + 1,
                        &line[0..span.start.col].replace("\t", "    "),
                        &line[span.start.col..].replace("\t", "    "),
                    )?;
                } else if last {
                    write!(
                        output,
                        "\x1b[2m{} |\x1b[22m {color}{}\x1b[0m{}",
                        span.start.ln + i + 1,
                        &line[0..span.end.col].replace("\t", "    "),
                        &line[span.end.col..].replace("\t", "    "),
                    )?;
                } else {
                    write!(
                        output,
                        "\x1b[2m{} |\x1b[22m \x1b[1;31m{}\x1b[0m",
                        span.start.ln + i + 1,
                        &line.replace("\t", "    "),
                    )?;
                }
            }
        }
        Ok(())
    }
}
