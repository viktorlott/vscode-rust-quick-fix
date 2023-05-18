use tower_lsp::lsp_types::{Range, TextDocumentContentChangeEvent, TextDocumentItem, Url};

#[derive(Debug)]
pub struct TextDocument {
    pub content: TextDocumentItem,
}

impl TextDocument {
    pub fn new(content: TextDocumentItem) -> Self {
        Self { content }
    }

    /// Commit the incomming changes to our store document
    /// This handles incremental changes
    ///
    /// TODO: Include history
    pub fn commit(mut self, changes: Vec<TextDocumentContentChangeEvent>, version: i32) -> Self {
        // Update the version
        self.content.version = version;
        for change in changes.into_iter().rev() {
            match change.range {
                Some(ref text_range) => {
                    let range = self.range_to_indices(text_range);
                    self.content.text.replace_range(range, change.text.as_str());
                }
                None => {
                    self.content.text = change.text;
                }
            }
        }
        self
    }

    /// Takes the incomming Range and translates it into the a document range
    fn range_to_indices(&self, text_range: &Range) -> std::ops::Range<usize> {
        let lines = self
            .content
            .text
            .char_indices()
            .fold(vec![0], |mut lines, (idx, char)| {
                if char == '\n' {
                    lines.push(idx + 1);
                }
                lines
            });

        let Range { start, end } = text_range;
        let start_idx = lines[start.line as usize] + start.character as usize;
        let end_idx = lines[end.line as usize] + end.character as usize;
        let content_length = self.content.text.len();

        if end_idx > content_length {
            start_idx..content_length
        } else if start_idx > end_idx {
            end_idx..start_idx
        } else {
            start_idx..end_idx
        }
    }

    /// This returns a text slice from a range
    pub fn select_range(&self, text_range: &Range) -> &str {
        &self.content.text[self.range_to_indices(text_range)]
    }

    /// Note that this returns a cloned `uri`
    pub fn get_uri(&self) -> Url {
        self.content.uri.clone()
    }

    pub fn get_version(&self) -> i32 {
        self.content.version
    }

    /// Note that this returns a cloned `language_id`
    pub fn get_language_id(&self) -> String {
        self.content.language_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, TextDocumentItem, Url};

    use super::TextDocument;

    #[test]
    fn parse_content_change() {
        let text_document = TextDocument::new(TextDocumentItem::new(
            Url::parse("file://somename").unwrap(),
            "rust".to_string(),
            3,
            "some\ncool over\nhere\nlast".to_string(),
        ));

        let content_change: TextDocumentContentChangeEvent = serde_json::from_str(
            r#"{
                "range": {
                    "start": {
                        "line": 2,
                        "character": 2
                    },
                    "end": {
                        "line": 3,
                        "character": 1
                    }
                },
                "rangeLength": 4,
                "text": "r"
        }"#,
        )
        .unwrap();

        let new_text_doc = text_document.commit(vec![content_change], 3);

        assert_eq!(
            new_text_doc.content.text,
            "some\ncool over\nherast".to_string()
        );
    }
}
