use std::io;

use crate::{diagnostic::Diagnostic, span::Span};

pub struct ErrorHandler {
    pub(crate) errors: Vec<Diagnostic>,
    pub(crate) warnings: Vec<Diagnostic>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn err(&mut self, message: impl ToString, span: Span) {
        self.errors.push(Diagnostic::new(message, span));
    }
    pub fn warn(&mut self, message: impl ToString, span: Span) {
        self.warnings.push(Diagnostic::new(message, span));
    }

    pub fn err_and(&mut self, message: impl ToString, span: Span) -> &mut Diagnostic {
        self.errors.push(Diagnostic::new(message, span));
        self.errors.last_mut().unwrap()
    }
    pub fn warn_and(&mut self, message: impl ToString, span: Span) -> &mut Diagnostic {
        self.warnings.push(Diagnostic::new(message, span));
        self.warnings.last_mut().unwrap()
    }

    pub fn did_error(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn write_to(
        &self,
        output: &mut impl io::Write,
        source: &str,
        config: &Config,
    ) -> io::Result<()> {
        for warn in &self.warnings {
            warn.write_to(
                output,
                source,
                config.warning_tag,
                config.warning_color,
                config.help_tag,
                config.help_color,
            )?;
        }
        for error in &self.errors {
            error.write_to(
                output,
                source,
                config.error_tag,
                config.error_color,
                config.help_tag,
                config.help_color,
            )?;
        }
        Ok(())
    }

    pub fn print(&self, source: &str, config: &Config) {
        self.write_to(&mut io::stdout(), source, config)
            .expect("failed to print to standard output.");
    }
}

pub struct Config {
    pub error_color: &'static str,
    pub error_tag: &'static str,
    pub warning_color: &'static str,
    pub warning_tag: &'static str,
    pub help_color: &'static str,
    pub help_tag: &'static str,
}
