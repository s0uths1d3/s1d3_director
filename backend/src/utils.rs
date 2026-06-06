/// 安全地将字符串截断到指定最大字节长度（在 UTF-8 字符边界处截断）
pub fn safe_truncate(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    // 在完整字符串上遍历字符边界，找到最后一个 <= max_bytes 的位置
    // 避免 s[..max_bytes] 切片（当 max_bytes 落在多字节字符中间时会 panic）
    match s.char_indices()
        .take_while(|(i, _)| *i <= max_bytes)
        .last()
    {
        Some((end, _)) => &s[..end],
        None => "",
    }
}
