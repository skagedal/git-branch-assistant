use anyhow::{Result, anyhow};
use dialoguer::{Select, theme::ColorfulTheme};

#[derive(Default, Clone)]
pub struct UserInterface;

#[derive(Clone)]
pub struct Choice<T> {
    identifier: T,
    label: String,
}

impl<T> Choice<T> {
    pub fn new(identifier: T, label: impl Into<String>) -> Self {
        Self {
            identifier,
            label: label.into(),
        }
    }
}

impl UserInterface {
    pub fn new() -> Self {
        Self
    }

    pub fn pick_one<T: Clone>(&self, message: &str, choices: &[Choice<T>]) -> Result<T> {
        if choices.is_empty() {
            return Err(anyhow!("no choices provided"));
        }

        let labels: Vec<&str> = choices.iter().map(|choice| choice.label.as_str()).collect();
        let theme = ColorfulTheme::default();
        let selection = Select::with_theme(&theme)
            .with_prompt(message)
            .items(&labels)
            .default(0)
            .interact_opt()?;

        match selection {
            Some(index) => Ok(choices[index].identifier.clone()),
            None => Err(anyhow!("no selection was made")),
        }
    }
}
