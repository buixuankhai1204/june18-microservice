pub fn increment_mixed_string(s: &str) -> Option<String> {
    // Tìm vị trí đầu tiên chứa số
    let pos = s.find(|c: char| c.is_ascii_digit())?;

    let (prefix, number_part) = s.split_at(pos);
    let num = number_part.parse::<u32>().ok()?;
    let new_number = format!("{:0width$}", num + 1, width = number_part.len());

    Some(format!("{}{}", prefix, new_number))
}
