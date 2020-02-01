use derive_more::Display;
use once_cell::sync::Lazy;
use regex;
use serde_json;
use sourcemap;
use std::{io, string::FromUtf8Error};

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "failed to read config file: {}", err)]
    FailedToReadConfigFile { err: io::Error },

    #[display(fmt = "failed to parse config file: {}", err)]
    FailedToParseConfigFile { err: serde_json::error::Error },

    #[display(fmt = "failed to parse module")]
    FailedToParseModule {},

    #[display(fmt = "failed to read module: {}", err)]
    FailedToReadModule { err: io::Error },

    #[display(fmt = "failed to emit module: {}", err)]
    FailedToEmitModule { err: io::Error },

    #[display(fmt = "failed to write sourcemap: {}", err)]
    FailedToWriteSourceMap { err: sourcemap::Error },

    #[display(fmt = "sourcemap is not utf8: {}", err)]
    SourceMapNotUtf8 { err: FromUtf8Error },

    #[display(fmt = "invalid regexp: {}: {}", regex, err)]
    InvalidRegex { regex: String, err: regex::Error },

    /* #[display = "generated code is not utf8: {}", err]
     * GeneratedCodeNotUtf8 { err: FromUtf8Error }, */
    /// This means `test` field in .swcrc file did not matched the compiling
    /// file.
    #[display = "unmatched"]
    Unmatched,
}

/// Returns true if `SWC_DEBUG` environment is set to `1` or `true`.
pub(crate) fn debug() -> bool {
    static DEBUG: Lazy<bool> = Lazy::new(|| match ::std::env::var("SWC_DEBUG") {
        Ok(ref v) if v == "1" || v.eq_ignore_ascii_case("true") => true,
        _ => false,
    });

    *DEBUG
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::FailedToReadConfigFile { err } => Some(err),
            Error::FailedToParseConfigFile { err } => Some(err),
            Error::FailedToParseModule {} => None,
            Error::FailedToReadModule { err } => Some(err),
            Error::FailedToEmitModule { err } => Some(err),
            Error::FailedToWriteSourceMap { err } => Some(err),
            Error::SourceMapNotUtf8 { err } => Some(err),
            Error::InvalidRegex { err, .. } => Some(err),
            Error::Unmatched => None,
        }
    }
}
