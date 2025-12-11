use std::collections::HashMap;
use anyhow::anyhow;
use fluent::FluentArgs;
use crate::localization::{read_ftl, Error, FluentBundle};

pub struct Locales {
    fallback: String,
    bundles: HashMap<String, FluentBundle>,
}

impl Locales {
    pub fn read_ftl(dir: &std::path::Path, fallback: String) -> Result<Self, Error> {
        let bundles = read_ftl(dir)?;

        // verify
        if !bundles.contains_key(&fallback) {
            return Err(anyhow!("fallback {} not found", fallback));
        }

        Ok(Locales { fallback, bundles })
    }

    pub fn resolve(&self, locale: &str, id: &str, args: Option<&FluentArgs>) -> Option<String> {
        let mut candidates = vec![locale];

        if let Some((language, _)) = locale.split_once('-') {
            candidates.push(language);
        }
        candidates.push(self.fallback.as_str());

        for candidate in candidates {
            let bundle = match self.bundles.get(candidate) {
                Some(bundle) => bundle,
                None => continue, // skips if no match
            };

            let message = match bundle.get_message(id) {
                Some(message) => message,
                None => continue, // skips if no match
            };
            let pattern = message.value()?;
            let formatted = bundle.format_pattern(pattern, args, &mut vec![]);

            return Some(formatted.into_owned())
        }

        None
    }
}