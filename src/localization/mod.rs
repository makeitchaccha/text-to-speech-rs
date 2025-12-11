use std::collections::HashMap;
use std::sync::Arc;
use anyhow::anyhow;

pub mod tts;

macro_rules! args {
    ( $(, $argname:ident: $argvalue:expr )* $(,)? ) => {{
        #[allow(unused_mut)]
        let mut args = fluent::FluentArgs::new();
        $( args.set(stringify!($argname), $argvalue); )*
        args
    }};
}

type Error = anyhow::Error;
type FluentBundle = fluent::bundle::FluentBundle<fluent::FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>;

fn read_ftl(dir: &std::path::Path) -> Result<HashMap<String, FluentBundle>, Error> {
    let bundles = dir.read_dir().map_err(|e| anyhow!("failed to read dir: {}", e))?
        .map(|file| read_single_ftl(&file?.path()))
        .collect::<Result<_, _>>()?;

    Ok(bundles)
}

// copied from serenity-rs/poise: https://github.com/serenity-rs/poise/blob/6b1bb9d/examples/fluent_localization/translation.rs
fn read_single_ftl(path: &std::path::Path) -> Result<(String, FluentBundle), Error> {
    // Extract locale from filename
    let locale = path.file_stem().ok_or(anyhow!("invalid .ftl filename"))?;
    let locale = locale.to_str().ok_or(anyhow!("invalid filename UTF-8"))?;

    // Load .ftl resource
    let file_contents = std::fs::read_to_string(path)?;
    let resource = fluent::FluentResource::try_new(file_contents)
        .map_err(|(_, e)| anyhow!("failed to parse {:?}: {:?}", path, e))?;

    // Associate .ftl resource with locale and bundle it
    let mut bundle = FluentBundle::new_concurrent(vec![locale
        .parse()
        .map_err(|e| anyhow!("invalid locale `{}`: {}", locale, e))?]);
    bundle
        .add_resource(resource)
        .map_err(|e| anyhow!("failed to add resource to bundle: {:?}", e))?;

    Ok((locale.to_string(), bundle))
}