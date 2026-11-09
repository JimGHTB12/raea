//! A layer's name: the component whose JSX drew it, read from the source at its use's loc.

/// The component a use's loc ("file:line:col", the col UTF-16 and 1-based, at its "<") names: `Badge` for
/// `<Badge x={1} />`, read from the source in `dir`. None when it can't be read.
pub fn component_at(dir: &std::path::Path, loc: &str) -> Option<String> {
    let mut it = loc.rsplitn(3, ':');
    let (col, line, file) = (it.next()?.parse::<usize>().ok()?, it.next()?.parse::<usize>().ok()?, it.next()?);
    let text = std::fs::read_to_string(dir.join(file)).ok()?;
    let row = text.lines().nth(line.checked_sub(1)?)?;
    // the UTF-16 column as a char offset
    let (mut units, mut start) = (0, None);
    for (i, c) in row.char_indices() {
        if units + 1 >= col {
            start = Some(i);
            break;
        }
        units += c.len_utf16();
    }
    let rest = row[start?..].strip_prefix('<')?;
    let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || matches!(c, '_' | '$' | '.')).collect();
    (!name.is_empty()).then_some(name)
}
