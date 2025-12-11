use std::collections::HashMap;
use anyhow::anyhow;
use fluent::FluentArgs;
use crate::localization::{read_ftl, Error, FluentBundle};

pub struct Locales {
    fallback: String,
    bundles: HashMap<String, FluentBundle>,
}

impl Locales {
    /// Loads all Fluent bundles for TTS announcements from the specified directory.
    ///
    /// The `fallback` locale serves as the final resort for all TTS messages
    /// when the requested profile locale is not available.
    pub fn read_ftl(dir: &std::path::Path, fallback: String) -> Result<Self, Error> {
        let bundles = read_ftl(dir)?;

        // verify
        if !bundles.contains_key(&fallback) {
            return Err(anyhow!("fallback {} not found", fallback));
        }

        Ok(Locales { fallback, bundles })
    }

    /// Resolves a localized message by searching through a cascading locale chain.
    ///
    /// For a given locale (e.g., "ja-JP") and a defined fallback (e.g., "en"),
    /// the search candidates are prioritized as: ["ja-JP", "ja", "en"].
    /// The first successfully resolved message is returned.
    pub fn resolve(&self, locale: &str, id: &str, args: Option<&FluentArgs>) -> Result<String, Error> {
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
            let pattern = message.value().ok_or(anyhow!("pattern has no value for id '{}'", id))?;
            let formatted = bundle.format_pattern(pattern, args, &mut vec![]);

            return Ok(formatted.into_owned())
        }

        Err(anyhow!("no fallback found for id '{}'", id))
    }
}