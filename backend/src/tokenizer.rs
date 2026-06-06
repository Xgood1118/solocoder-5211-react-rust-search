pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_en = String::new();

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if is_ascii_alphanumeric(c) {
            current_en.push(c.to_ascii_lowercase());
        } else {
            if !current_en.is_empty() {
                tokens.push(std::mem::take(&mut current_en));
            }

            if is_cjk(c) {
                if i + 1 < chars.len() && is_cjk(chars[i + 1]) {
                    let mut bigram = String::new();
                    bigram.push(c);
                    bigram.push(chars[i + 1]);
                    tokens.push(bigram);
                }
            }
        }
        i += 1;
    }

    if !current_en.is_empty() {
        tokens.push(current_en);
    }

    tokens
}

pub fn tokenize_phrase(text: &str) -> Vec<String> {
    tokenize(text)
}

fn is_ascii_alphanumeric(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x4E00..=0x9FFF |
        0x3400..=0x4DBF |
        0x3000..=0x303F |
        0xFF00..=0xFFEF
    )
}

pub fn is_chinese_char(c: char) -> bool {
    (c as u32) >= 0x4E00 && (c as u32) <= 0x9FFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_english() {
        let tokens = tokenize("hello world");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_chinese() {
        let tokens = tokenize("你好世界");
        assert_eq!(tokens, vec!["你好", "好世", "世界"]);
    }

    #[test]
    fn test_tokenize_mixed() {
        let tokens = tokenize("rust 内存管理");
        assert_eq!(tokens, vec!["rust", "内存", "存管", "管理"]);
    }

    #[test]
    fn test_tokenize_punctuation() {
        let tokens = tokenize("hello, world!");
        assert_eq!(tokens, vec!["hello", "world"]);
    }
}
