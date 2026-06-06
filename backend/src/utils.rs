/// 安全地将字符串截断到指定最大字节长度（在 UTF-8 字符边界处截断）
pub fn safe_truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    // 在 max_bytes 范围内找到最后一个字符边界
    match s[..max_bytes].char_indices().last() {
        Some((end, _)) => &s[..end],
        None => "", // 空字符串或 max_bytes == 0
    }
}
