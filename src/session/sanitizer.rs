use regex::Regex;
use std::sync::LazyLock;

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {Regex::new(r"https?://\S+").unwrap() });
static EMOJI_REGEX: LazyLock<Regex> = LazyLock::new(|| {Regex::new(r"<a?:(\w+):\d+>").unwrap() });
static CODE_BLOCK_REGEX: LazyLock<Regex> = LazyLock::new(|| {Regex::new(r"```(?:\w*\n)?(.*?)```").unwrap() });

pub fn sanitize(content: &str, limit: usize) -> String {
    let mut text = content.to_string();

    text = CODE_BLOCK_REGEX.replace_all(&text, "code block").to_string();

    text = URL_REGEX.replace_all(&text, "URL").to_string();

    text = EMOJI_REGEX.replace_all(&text, "EMOJI").to_string();

    text.chars().take(limit).collect()
}