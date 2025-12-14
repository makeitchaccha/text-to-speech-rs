use std::collections::HashMap;
use anyhow::anyhow;
use fluent::FluentArgs;
use include_dir::{include_dir, Dir};

type Error = anyhow::Error;
type FluentBundle = fluent::bundle::FluentBundle<fluent::FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>;

const TTS_LOCALES: Dir = include_dir!("$CARGO_MANIFEST_DIR/locales/tts");

pub fn load_tts_locales(fallback: &str) -> Result<Locales, Error> {
    load_from_static_dir(TTS_LOCALES, LocaleSearchPolicy::new_cascading(fallback.to_owned(), '-'))
}

fn load_from_static_dir(dir: Dir, policy: LocaleSearchPolicy) -> Result<Locales, Error> {
    let mut bundles = HashMap::new();

    for file in dir.files() {
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

    Locales::new_with_bundles(policy, bundles)
}

enum LocaleMatchingMode {
    Exact,
    Cascading { delimiter: char }
}

pub struct LocaleSearchPolicy {
    fallback: String,
    mode: LocaleMatchingMode,
}

impl LocaleSearchPolicy {
    fn new_cascading(fallback: String, delimiter: char) -> Self {
        Self {
            fallback,
            mode: LocaleMatchingMode::Cascading { delimiter }
        }
    }

    fn new_exact(fallback: String) -> Self {
        Self {
            fallback,
            mode: LocaleMatchingMode::Exact,
        }
    }

    fn generate_candidates<'a>(&'a self, locale: &'a str) -> impl Iterator<Item=&'a str> {
        let mut candidates = Vec::new();
        candidates.push(locale);

        if let LocaleMatchingMode::Cascading { delimiter } = self.mode {
            if let Some((language, _)) = locale.split_once(delimiter) {
                candidates.push(language)
            }
        }

        if !candidates.contains(&self.fallback.as_str()) {
            candidates.push(self.fallback.as_str());
        }

        candidates.into_iter()
    }
}

pub struct Locales {
    search_policy: LocaleSearchPolicy,
    bundles: HashMap<String, FluentBundle>,
}

impl Locales {
    pub fn new_with_bundles(search_policy: LocaleSearchPolicy, bundles: HashMap<String, FluentBundle>) -> Result<Self, Error> {
        if !bundles.contains_key(&search_policy.fallback) {
            return Err(anyhow!("fallback locale {} not found", &search_policy.fallback));
        }

        Ok(Self { search_policy, bundles })
    }

    /// Resolves a localized message by searching through candidates according to the configured search policy.
    ///
    /// For a given locale (e.g., "ja-JP") and a defined fallback (e.g., "en"),
    /// the search candidates are based on the search policy (cascading or exact).
    /// The first successfully resolved message is returned.
    pub fn resolve(&self, locale: &str, id: &str, args: Option<&FluentArgs>) -> Result<String, Error> {
        for candidate in self.search_policy.generate_candidates(locale) {
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

#[cfg(test)]
mod tests {
    use super::{LocaleMatchingMode, LocaleSearchPolicy};

    fn create_policy(fallback: &str, mode: LocaleMatchingMode) -> LocaleSearchPolicy {
        LocaleSearchPolicy {
            fallback: fallback.to_string(),
            mode,
        }
    }

    #[test]
    fn test_cascading_with_region_match() {
        let policy = create_policy("ja", LocaleMatchingMode::Cascading { delimiter: '-' });
        let candidates: Vec<_> = policy.generate_candidates("en-US").collect();

        assert_eq!(candidates, vec!["en-US", "en", "ja"]);
    }

    #[test]
    fn test_cascading_base_language_only() {
        let policy = create_policy("ja", LocaleMatchingMode::Cascading { delimiter: '-' });
        let candidates: Vec<_> = policy.generate_candidates("en").collect();

        assert_eq!(candidates, vec!["en", "ja"]);
    }

    #[test]
    fn test_cascading_locale_is_ultimate_fallback() {
        let policy = create_policy("en", LocaleMatchingMode::Cascading { delimiter: '-' });
        let candidates: Vec<_> = policy.generate_candidates("en").collect();

        assert_eq!(candidates, vec!["en"]);
    }

    #[test]
    fn test_cascading_needs_fallback_no_prefix() {
        let policy = create_policy("en", LocaleMatchingMode::Cascading { delimiter: '-' });
        let candidates: Vec<_> = policy.generate_candidates("fr").collect();

        assert_eq!(candidates, vec!["fr", "en"]);
    }

    #[test]
    fn test_exact_match_no_prefix_added() {
        let policy = create_policy("en", LocaleMatchingMode::Exact);
        let candidates: Vec<_> = policy.generate_candidates("zh-Hans").collect();

        assert_eq!(candidates, vec!["zh-Hans", "en"]);
    }

    #[test]
    fn test_exact_match_locale_is_ultimate_fallback() {
        let policy = create_policy("ja", LocaleMatchingMode::Exact);
        let candidates: Vec<_> = policy.generate_candidates("ja").collect();

        assert_eq!(candidates, vec!["ja"]);
    }
}