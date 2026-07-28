#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftMode {
    Auto,
    Structured,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftField {
    Type,
    Scope,
    Breaking,
    Summary,
    Body,
    Trailers,
}

impl DraftField {
    pub const ALL: [Self; 6] = [
        Self::Type,
        Self::Scope,
        Self::Breaking,
        Self::Summary,
        Self::Body,
        Self::Trailers,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Scope => "scope",
            Self::Breaking => "breaking",
            Self::Summary => "summary",
            Self::Body => "body",
            Self::Trailers => "trailers",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitDraft {
    pub mode: DraftMode,
    pub commit_type: String,
    pub scope: String,
    pub breaking: bool,
    pub summary: String,
    pub body: String,
    pub trailers: String,
}

impl Default for CommitDraft {
    fn default() -> Self {
        Self {
            mode: DraftMode::Auto,
            commit_type: "feat".to_string(),
            scope: String::new(),
            breaking: false,
            summary: String::new(),
            body: String::new(),
            trailers: String::new(),
        }
    }
}

impl CommitDraft {
    pub const AUTO_MESSAGE: &'static str = "Auto commit";

    pub fn with_message(message: String) -> Self {
        Self {
            mode: DraftMode::Custom,
            summary: message,
            ..Self::default()
        }
    }

    pub fn message(&self) -> String {
        match self.mode {
            DraftMode::Auto => return Self::AUTO_MESSAGE.to_string(),
            DraftMode::Custom => return self.summary.clone(),
            DraftMode::Structured => {}
        }
        let scope = (!self.scope.trim().is_empty()).then(|| format!("({})", self.scope.trim()));
        let breaking = if self.breaking { "!" } else { "" };
        let mut message = format!(
            "{}{}{}: {}",
            self.commit_type.trim(),
            scope.unwrap_or_default(),
            breaking,
            self.summary.trim()
        );
        if !self.body.trim().is_empty() {
            message.push_str("\n\n");
            message.push_str(self.body.trim());
        }
        if !self.trailers.trim().is_empty() {
            message.push_str("\n\n");
            message.push_str(self.trailers.trim());
        }
        message
    }

    pub fn validation_error(&self) -> Option<&'static str> {
        if self.mode == DraftMode::Auto {
            return None;
        }
        if self.mode == DraftMode::Custom {
            return self
                .summary
                .trim()
                .is_empty()
                .then_some("A custom commit message is required.");
        }
        if self.commit_type.trim().is_empty() {
            Some("A commit type is required.")
        } else if self.summary.trim().is_empty() {
            Some("A commit summary is required.")
        } else if self.summary.contains('\n') {
            Some("The commit summary must stay on one line.")
        } else {
            None
        }
    }

    pub fn field(&self, field: DraftField) -> &str {
        match field {
            DraftField::Type => &self.commit_type,
            DraftField::Scope => &self.scope,
            DraftField::Breaking => "",
            DraftField::Summary => &self.summary,
            DraftField::Body => &self.body,
            DraftField::Trailers => &self.trailers,
        }
    }

    pub fn field_mut(&mut self, field: DraftField) -> Option<&mut String> {
        match field {
            DraftField::Type => Some(&mut self.commit_type),
            DraftField::Scope => Some(&mut self.scope),
            DraftField::Breaking => None,
            DraftField::Summary => Some(&mut self.summary),
            DraftField::Body => Some(&mut self.body),
            DraftField::Trailers => Some(&mut self.trailers),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_message_is_explicit_and_valid() {
        let draft = CommitDraft::default();
        assert_eq!(draft.message(), "Auto commit");
        assert_eq!(draft.validation_error(), None);
    }

    #[test]
    fn structured_message_includes_all_optional_parts() {
        let draft = CommitDraft {
            mode: DraftMode::Structured,
            commit_type: "feat".to_string(),
            scope: "ui".to_string(),
            breaking: true,
            summary: "add review".to_string(),
            body: "Explain why.".to_string(),
            trailers: "Refs: #12".to_string(),
        };
        assert_eq!(
            draft.message(),
            "feat(ui)!: add review\n\nExplain why.\n\nRefs: #12"
        );
    }

    #[test]
    fn custom_message_remains_verbatim() {
        let draft = CommitDraft::with_message("fix: preserve this exactly".to_string());
        assert_eq!(draft.message(), "fix: preserve this exactly");
    }
}
