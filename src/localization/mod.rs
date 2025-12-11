pub mod tts;

type Error = anyhow::Error;
type FluentBundle = fluent::bundle::FluentBundle<fluent::FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>;
