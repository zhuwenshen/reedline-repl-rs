use crate::error::*;
use std::collections::HashMap;

/// Command parameter
#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) default: Option<String>,
    pub(crate) help_summary: Option<String>,
    pub(crate) allowed_values: HashMap<String, Option<String>>,
}

impl Parameter {
    /// Create a new command parameter with the given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            required: false,
            allowed_values: HashMap::new(),
            default: None,
            help_summary: None,
        }
    }

    /// Set whether the parameter is required, default is not required.
    /// Note that you cannot have a required parameter after a non-required one
    pub fn set_required(mut self, required: bool) -> Result<Self> {
        if self.default.is_some() {
            return Err(Error::IllegalRequiredError(self.name));
        }
        self.required = required;

        Ok(self)
    }

    /// Sets help summary
    pub fn with_help(mut self, help_summary: &str) -> Self {
        self.help_summary = Some(help_summary.to_string());
        self
    }

    /// Add an allowed value the parameter may take with optional help text
    pub fn add_allowed_value(mut self, allowed_value: &str, help_summary: Option<&str>) -> Self {
        self.allowed_values.insert(
            allowed_value.to_string(),
            help_summary.map(|s| s.to_string()),
        );
        self
    }

    /// Set a default for an optional parameter.
    /// Note that you can't have a default for a required parameter
    pub fn set_default(mut self, default: &str) -> Result<Self> {
        if self.required {
            return Err(Error::IllegalDefaultError(self.name));
        }
        self.default = Some(default.to_string());

        Ok(self)
    }
}
