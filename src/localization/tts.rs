use std::collections::HashMap;
use anyhow::anyhow;
use fluent::FluentArgs;
use crate::localization::{Error, FluentBundle};
use include_dir::{include_dir, Dir};

const LOCALES: Dir = include_dir!("$CARGO_MANIFEST_DIR/locales/tts");

pub fn load_from_static_dir(fallback: &str) -> Result<Locales, Error> {
    let mut bundles = HashMap::new();

    for file in LOCALES.files() {
        let locale = file.path()
            .file_stem().ok_or(anyhow!("Invalid file name: '{}'", file.path().display()))?
            .to_str().ok_or(anyhow!("Invalid unicode filename"))?;

        let resource = fluent::FluentResource::try_new(file.contents_utf8().ok_or(anyhow!("Invalid file contents"))?.to_owned())
            .map_err(|(_, e)| anyhow!("failed to parse {:?}: {:?}", file.path(), e))?;

        let mut bundle = FluentBundle::new_concurrent(vec![locale
            .parse()
            .map_err(|e| anyhow!("invalid locale `{}`: {}", locale, e))?]);
        bundle
            .add_resource(resource)
            .map_err(|e| anyhow!("failed to add resource to bundle: {:?}", e))?;

        bundles.insert(locale.to_owned(), bundle);
    }

    Locales::new_with_bundles(fallback.to_string(), bundles)
}

pub struct Locales {
    fallback: String,
    bundles: HashMap<String, FluentBundle>,
}

impl Locales {
    pub fn new_with_bundles(fallback: String, bundles: HashMap<String, FluentBundle>) -> Result<Self, Error> {
        if !bundles.contains_key(&fallback) {
            return Err(anyhow!("fallback locale {} not found", fallback));
        }

        Ok(Self { fallback, bundles })
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
        if !candidates.contains(&self.fallback.as_str()) {
            candidates.push(self.fallback.as_str());
        }

        for candidate in candidates {
            let bundle = match self.bundles.get(candidate) {
                Some(bundle) => bundle,
                None => continue, // skips if no match
            };

            let message = match bundle.get_message(id) {
                Some(message) => message,
                None => continue, // skips if no match
            };
            let pattern = message.value().ok_or(anyhow!("message '{}' exists but has no value pattern", id))?;
            let formatted = bundle.format_pattern(pattern, args, &mut vec![]);

            return Ok(formatted.into_owned())
        }

        Err(anyhow!("no fallback found for id '{}'", id))
    }
}