/// 转义字符映射
const ESCAPE_SEQUENCES: &[(u8, &str)] = &[
    (b'\\', "\\\\"),
    (b'"', "\\\""),
    (b'\n', "\\n"),
    (b'\r', "\\r"),
    (b'\t', "\\t"),
];

/// 转义字符集合
const ESCAPE_BYTES: &[u8] = &[b'\\', b'"', b'\n', b'\r', b'\t'];

/// 反转义字符映射
fn unescape_char(byte: u8) -> Option<char> {
    match byte {
        b'"' => Some('"'),
        b'\\' => Some('\\'),
        b'n' => Some('\n'),
        b'r' => Some('\r'),
        b't' => Some('\t'),
        _ => None,
    }
}

/// 反转义字符串
///
/// # Arguments
///
/// * `s`: 字符串
///
/// returns: String
///
/// # Examples
///
/// ```
///
/// ```
pub fn unescape_string(s: &str) -> String {
    // 快速路径
    if !s.as_bytes().contains(&b'\\') {
        return s.to_string();
    }

    let mut out = String::with_capacity(s.len());
    let mut rest = s;

    while let Some(pos) = rest.find('\\') {
        out.push_str(&rest[..pos]);

        // 处理反斜杠
        let after = &rest[pos + 1..];

        if after.is_empty() {
            out.push('\\');
            return out;
        }

        let b = after.as_bytes()[0];
        match unescape_char(b) {
            Some(ch) => out.push(ch),
            None => {
                // 未知转义序列，保留原样
                out.push('\\');
                out.push(b as char);
            }
        }

        rest = &after[1..];
    }

    out.push_str(rest);
    out
}

/// 转义字符串
///
/// # Arguments
///
/// * `s`: 字符串
///
/// returns: String
///
/// # Examples
///
/// ```
///
/// ```
pub fn escape_string(s: &str) -> String {
    let bytes = s.as_bytes();

    // 快速路径
    if !bytes.iter().any(|&b| ESCAPE_BYTES.contains(&b)) {
        return s.to_string();
    }

    let mut out = String::with_capacity(s.len());
    let mut last = 0usize;
    let mut i = 0usize;

    while i < bytes.len() {
        let b = bytes[i];
        let esc = ESCAPE_SEQUENCES
            .iter()
            .find(|&&(byte, _)| byte == b)
            .map(|&(_, seq)| seq);

        if let Some(rep) = esc {
            out.push_str(&s[last..i]);
            out.push_str(rep);
            i += 1;
            last = i;
        } else {
            i += 1;
        }
    }

    out.push_str(&s[last..]);
    out
}
