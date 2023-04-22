#[cfg(feature = "discord")]
use regex::Regex;
use rustwemoji::get;

#[cfg(feature = "discord")]
const RE_DISCORD_EMOJI: &str = r"<a?:[a-zA-Z0-9_]+:([0-9]{17,19})>";

/// Tokens parsed
#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    /// Text token
    Text(String),
    /// Emoji token(bytes of png)
    Emoji(Vec<u8>),
    #[cfg(feature = "discord")]
    /// Custom emoji token(url)
    CustomEmoji(String),
}

impl Token {
    pub fn new_text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }
    pub fn new_emoji(s: impl Into<Vec<u8>>) -> Self {
        Self::Emoji(s.into())
    }
    #[cfg(feature = "discord")]
    pub fn new_custom_emoji(s: String) -> Self {
        let s = format!("https://cdn.discordapp.com/emojis/{}.png?size=96", s);
        Self::CustomEmoji(s)
    }
}

#[cfg(not(feature = "discord"))]
fn raw_parse(s: String) -> Vec<Token> {
    raw_parse_emoji(s)
}

#[cfg(not(feature = "async"))]
pub fn parse(s: String) -> Vec<Token> {
    raw_parse(s)
}

#[cfg(feature = "tokio")]
/// Parse a string to tokens
pub async fn parse(s: String) -> Result<Vec<Token>, tokio::task::JoinError> {
    tokio::task::spawn(async { raw_parse(s) }).await
}

#[cfg(feature = "async-std")]
/// Parse a string to tokens
pub async fn parse(s: String) -> Vec<Token> {
    async_std::task::spawn(async { raw_parse(s) }).await
}

fn raw_parse_emoji(s: String) -> Vec<Token> {
    let s = s.chars();
    s.map(|f| f.to_string())
        .map(|f| {
            if let Some(v) = get(&f) {
                Token::new_emoji(v)
            } else {
                Token::new_text(f)
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(feature = "discord")]
fn raw_parse(s: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let re = Regex::new(RE_DISCORD_EMOJI).unwrap();
    let mut last = 0;
    for m in re.find_iter(&s) {
        let (start, end) = (m.range().start, m.range().end);
        let text = s[last..start].to_string();
        let emoji = s[start..end].to_string();
        let id = re
            .captures(&emoji)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_string();
        tokens.extend(raw_parse_emoji(text));
        tokens.push(Token::new_custom_emoji(id));
        last = end;
    }
    let text = s[last..].to_string();
    tokens.extend(raw_parse_emoji(text));
    tokens
}

#[cfg(test)]
mod test {
    #[cfg(all(feature = "discord", feature = "async"))]
    use super::*;
    #[cfg(all(feature = "discord", feature = "async-std"))]
    #[async_std::test]
    async fn test_parse() {
        let s = "Hello <a:pepega:123456789012345678> World".to_string();
        let tokens = parse(s).await;
        assert_eq!(
            tokens,
            vec![
                Token::new_text("H"),
                Token::new_text("e"),
                Token::new_text("l"),
                Token::new_text("l"),
                Token::new_text("o"),
                Token::new_text(" "),
                Token::new_custom_emoji("123456789012345678".to_string()),
                Token::new_text(" "),
                Token::new_text("W"),
                Token::new_text("o"),
                Token::new_text("r"),
                Token::new_text("l"),
                Token::new_text("d"),
            ]
        );
    }
    #[cfg(all(feature = "discord", feature = "tokio"))]
    #[tokio::test]
    async fn test_parse() {
        let s = "Hello <a:pepega:123456789012345678> World".to_string();
        let tokens = parse(s).await.unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::new_text("H"),
                Token::new_text("e"),
                Token::new_text("l"),
                Token::new_text("l"),
                Token::new_text("o"),
                Token::new_text(" "),
                Token::new_custom_emoji("123456789012345678".to_string()),
                Token::new_text(" "),
                Token::new_text("W"),
                Token::new_text("o"),
                Token::new_text("r"),
                Token::new_text("l"),
                Token::new_text("d"),
            ]
        );
    }
}

// Will make a compile error with both async-syd and tokio enabled
#[cfg(all(feature = "async-std", feature = "tokio"))]
compile_error!("You can only enable one of the async features");
