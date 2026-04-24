//! # Quran Delimited Parser
//!
//! ### Architectural Design Decision: Concrete Parser Implementation
//! Implements `ScriptureParser` specifically for pipe-delimited (`|`)
//! texts where `Column 1 = Macro`, `Column 2 = LexKey`, and `Column 3 = Text`.

use crate::fsi::models::{Coordinate, LexKey, MacroID, NamespaceID, WorkID};
use crate::parsers::{ParsedEntry, ScriptureParser};
use crate::utils::errors::ScriptureError;

pub struct QuranPipeParser {
    pub work_id: WorkID,
    pub namespace_id: NamespaceID,
}

impl ScriptureParser for QuranPipeParser {
    /// ## `parse`
    ///
    /// ### Technical Context: Delimiter Extraction
    /// Iterates line-by-line, splitting on the `|` character to build the 3D FSI Coordinate.
    fn parse(&self, raw_content: &str) -> Result<Vec<ParsedEntry>, ScriptureError> {
        let mut entries = Vec::new();

        for (line_num, line) in raw_content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines and copyright/comment blocks
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 3 {
                return Err(ScriptureError::ParseError(format!(
                    "Invalid format at line {}",
                    line_num + 1
                )));
            }

            let macro_id = parts[0].parse::<i32>().unwrap_or(0);
            let lex_key = parts[1].to_string();
            let text = parts[2..].join("|"); // Join in case text contains pipes

            let coordinate = Coordinate {
                work_id: self.work_id,
                macro_id: MacroID(macro_id),
                lex_key: LexKey(lex_key),
            };

            entries.push(ParsedEntry {
                coordinate,
                text: text.trim().to_string(),
                namespace_id: self.namespace_id,
            });
        }
        Ok(entries)
    }
}
