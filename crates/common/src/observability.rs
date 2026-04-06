use serde_json::Value;

const REDACTED: &str = "[REDACTED]";
const EXACT_SENSITIVE_KEYS: &[&str] = &["token", "password", "secret", "cookie", "bearer"];
const SUBSTRING_SENSITIVE_KEYS: &[&str] = &[
    "api_key",
    "apikey",
    "access_token",
    "refresh_token",
    "authorization",
    "set_cookie",
    "secret_key",
    "client_secret",
    "private_key",
    "session_token",
    "id_token",
    "auth_token",
];
const TOKEN_PREFIXES: &[&str] = &[
    "sk-",
    "pk-lf-",
    "xoxb-",
    "xapp-",
    "xoxp-",
    "ghp_",
    "ghu_",
    "github_pat_",
];

#[must_use]
pub fn truncate_text(input: &str, max_bytes: usize) -> String {
    if input.len() <= max_bytes {
        return input.to_string();
    }

    let original_len = input.len();
    let mut end = max_bytes;
    while end > 0 && !input.is_char_boundary(end) {
        end -= 1;
    }

    format!(
        "{}\n\n[truncated — {original_len} bytes total]",
        &input[..end]
    )
}

#[must_use]
pub fn is_sensitive_key(key: &str) -> bool {
    let normalized = key.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    EXACT_SENSITIVE_KEYS
        .iter()
        .any(|needle| normalized == *needle)
        || SUBSTRING_SENSITIVE_KEYS
            .iter()
            .any(|needle| normalized.contains(needle))
}

#[must_use]
pub fn redact_json_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut redacted = serde_json::Map::with_capacity(map.len());
            for (key, value) in map {
                if is_sensitive_key(key) {
                    redacted.insert(key.clone(), Value::String(REDACTED.to_string()));
                } else {
                    redacted.insert(key.clone(), redact_json_value(value));
                }
            }
            Value::Object(redacted)
        },
        Value::Array(values) => Value::Array(values.iter().map(redact_json_value).collect()),
        _ => value.clone(),
    }
}

#[must_use]
pub fn sanitize_json_for_observability(value: &Value, max_bytes: usize, redact: bool) -> Value {
    let prepared = if redact {
        redact_json_value(value)
    } else {
        value.clone()
    };

    match serde_json::to_string(&prepared) {
        Ok(serialized) if serialized.len() <= max_bytes => prepared,
        Ok(serialized) => Value::String(truncate_text(&serialized, max_bytes)),
        Err(_) => Value::String("[unserializable json]".to_string()),
    }
}

#[must_use]
pub fn sanitize_text_for_observability(input: &str, max_bytes: usize, redact: bool) -> String {
    let text = if redact {
        redact_text(input)
    } else {
        input.to_string()
    };
    truncate_text(&text, max_bytes)
}

#[must_use]
pub fn redact_text(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    for segment in input.split_inclusive('\n') {
        let (line, newline) = if let Some(stripped) = segment.strip_suffix('\n') {
            (stripped, "\n")
        } else {
            (segment, "")
        };
        result.push_str(&redact_line(line));
        result.push_str(newline);
    }

    result
}

fn redact_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return line.to_string();
    }

    if let Some(redacted) = redact_inline_assignments(line) {
        return redacted;
    }

    let lower = line.to_ascii_lowercase();
    if let Some(index) = lower.find("authorization:") {
        return format!("{}Authorization: {REDACTED}", &line[..index]);
    }
    if let Some(index) = lower.find("cookie:") {
        return format!("{}Cookie: {REDACTED}", &line[..index]);
    }
    if let Some(index) = lower.find("set-cookie:") {
        return format!("{}Set-Cookie: {REDACTED}", &line[..index]);
    }

    let bearer_redacted = redact_bearer_token(line);
    redact_prefixed_tokens(&bearer_redacted)
}

fn redact_inline_assignments(line: &str) -> Option<String> {
    let mut redactions = Vec::new();

    for (index, ch) in line.char_indices() {
        if !matches!(ch, ':' | '=') {
            continue;
        }

        let Some((key_start, key_end)) = assignment_key_bounds(line, index) else {
            continue;
        };
        let key = line[key_start..key_end].trim_matches(|c| matches!(c, '"' | '\'' | '`'));
        if !is_sensitive_key(key) {
            continue;
        }

        let Some((value_start, value_end, replacement)) = assignment_value_bounds(line, index)
        else {
            continue;
        };
        redactions.push((value_start, value_end, replacement));
    }

    if redactions.is_empty() {
        return None;
    }

    redactions.sort_by(|left, right| left.0.cmp(&right.0));
    let mut collapsed = Vec::with_capacity(redactions.len());
    for (start, end, replacement) in redactions {
        if collapsed
            .last()
            .is_some_and(|(last_start, last_end, _): &(usize, usize, String)| {
                start < *last_end || end <= *last_start
            })
        {
            continue;
        }
        collapsed.push((start, end, replacement.to_string()));
    }

    let mut output = line.to_string();
    for (start, end, replacement) in collapsed.into_iter().rev() {
        output.replace_range(start..end, &replacement);
    }
    Some(output)
}

fn assignment_key_bounds(line: &str, separator_index: usize) -> Option<(usize, usize)> {
    let mut key_end = separator_index;
    while key_end > 0
        && line[..key_end]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
    {
        key_end = previous_char_boundary(line, key_end)?;
    }
    if key_end == 0 {
        return None;
    }

    let mut key_start = key_end;
    while key_start > 0 {
        let previous_index = previous_char_boundary(line, key_start)?;
        let ch = line[previous_index..key_start].chars().next()?;
        if is_assignment_delimiter(ch) {
            break;
        }
        key_start = previous_index;
    }

    (key_start < key_end).then_some((key_start, key_end))
}

fn assignment_value_bounds(
    line: &str,
    separator_index: usize,
) -> Option<(usize, usize, &'static str)> {
    let separator_end = separator_index + 1;
    let mut value_start = separator_end;
    while value_start < line.len()
        && line[value_start..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
    {
        value_start += line[value_start..].chars().next()?.len_utf8();
    }

    if value_start >= line.len() {
        return Some((separator_end, separator_end, REDACTED));
    }

    let first_char = line[value_start..].chars().next()?;
    if matches!(first_char, '"' | '\'') {
        let quote = first_char;
        let mut value_end = value_start + quote.len_utf8();
        let mut escaped = false;
        for (offset, ch) in line[value_end..].char_indices() {
            value_end = value_end.max(value_start + quote.len_utf8() + offset + ch.len_utf8());
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                let end = value_start + quote.len_utf8() + offset + ch.len_utf8();
                return Some((value_start, end, "\"[REDACTED]\""));
            }
        }
        return Some((value_start, line.len(), "\"[REDACTED]\""));
    }

    let mut value_end = value_start;
    while value_end < line.len() {
        let ch = line[value_end..].chars().next()?;
        if is_value_delimiter(ch) {
            break;
        }
        value_end += ch.len_utf8();
    }

    Some((value_start, value_end, REDACTED))
}

fn previous_char_boundary(line: &str, index: usize) -> Option<usize> {
    line[..index]
        .char_indices()
        .next_back()
        .map(|(offset, _)| offset)
}

fn is_assignment_delimiter(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, ',' | ';' | '&' | '{' | '}' | '[' | ']' | '(' | ')')
}

fn is_value_delimiter(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, ',' | ';' | '&' | '}' | ']' | ')')
}

fn redact_bearer_token(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    let Some(index) = lower.find("bearer ") else {
        return line.to_string();
    };

    let token_start = index + "bearer ".len();
    let token_len = line[token_start..]
        .chars()
        .take_while(|ch| is_token_char(*ch))
        .map(char::len_utf8)
        .sum::<usize>();
    if token_len == 0 {
        return line.to_string();
    }

    let token_end = token_start + token_len;
    format!("{}Bearer {REDACTED}{}", &line[..index], &line[token_end..])
}

fn redact_prefixed_tokens(line: &str) -> String {
    let mut current = line.to_string();
    for prefix in TOKEN_PREFIXES {
        current = redact_token_prefix(&current, prefix);
    }
    current
}

fn redact_token_prefix(line: &str, prefix: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut rest = line;

    while let Some(index) = rest.find(prefix) {
        let token_start = index;
        let token_body = &rest[token_start + prefix.len()..];
        let token_len = token_body
            .chars()
            .take_while(|ch| is_token_char(*ch))
            .map(char::len_utf8)
            .sum::<usize>();

        if token_len < 8 {
            result.push_str(&rest[..token_start + prefix.len()]);
            rest = &rest[token_start + prefix.len()..];
            continue;
        }

        let token_end = token_start + prefix.len() + token_len;
        result.push_str(&rest[..token_start]);
        result.push_str(REDACTED);
        rest = &rest[token_end..];
    }

    result.push_str(rest);
    result
}

fn is_token_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_json_replaces_sensitive_keys() {
        let input = serde_json::json!({
            "api_key": "sk-123",
            "nested": {
                "access_token": "abc",
                "safe": "ok"
            }
        });

        let redacted = redact_json_value(&input);

        assert_eq!(redacted["api_key"], REDACTED);
        assert_eq!(redacted["nested"]["access_token"], REDACTED);
        assert_eq!(redacted["nested"]["safe"], "ok");
    }

    #[test]
    fn redact_text_replaces_sensitive_assignments() {
        let input = "api_key = sk-secret\nAuthorization: Bearer abc123456789\nsafe = ok";
        let output = redact_text(input);

        assert!(output.contains("api_key = [REDACTED]"));
        assert!(output.contains("Authorization: [REDACTED]"));
        assert!(output.contains("safe = ok"));
    }

    #[test]
    fn redact_text_replaces_known_prefix_tokens() {
        let input = "Langfuse pk-lf-1234567890abcdef and OpenAI sk-abcdefghijklmnop";
        let output = redact_text(input);

        assert!(!output.contains("pk-lf-1234567890abcdef"));
        assert!(!output.contains("sk-abcdefghijklmnop"));
        assert!(output.contains(REDACTED));
    }

    #[test]
    fn is_sensitive_key_does_not_flag_token_count() {
        assert!(!is_sensitive_key("token_count"));
        assert!(!is_sensitive_key("output_tokens"));
        assert!(is_sensitive_key("access_token"));
        assert!(is_sensitive_key("secret_key"));
    }

    #[test]
    fn redact_text_scans_past_first_assignment_in_line() {
        let input = "safe = ok password = hunter2";
        let output = redact_text(input);

        assert!(output.contains("safe = ok"));
        assert!(output.contains("password = [REDACTED]"));
        assert!(!output.contains("hunter2"));
    }

    #[test]
    fn redact_text_preserves_json_structure_for_sensitive_keys() {
        let input = r#"{"token":"secret","token_count":42}"#;
        let output = redact_text(input);

        assert_eq!(output, r#"{"token":"[REDACTED]","token_count":42}"#);
    }

    #[test]
    fn sanitize_json_truncates_large_payloads() {
        let input = serde_json::json!({
            "value": "x".repeat(200)
        });

        let output = sanitize_json_for_observability(&input, 32, false);
        let output = output.as_str().expect("string output");

        assert!(output.contains("[truncated"));
    }
}
