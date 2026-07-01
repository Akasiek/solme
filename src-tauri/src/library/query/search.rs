pub(crate) fn search_query(input: &str) -> Option<String> {
    let terms = input
        .split_whitespace()
        .map(|term| {
            term.chars()
                .filter(|character| character.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|term| !term.is_empty())
        .map(|term| format!("{term}*"))
        .collect::<Vec<_>>();

    (!terms.is_empty()).then(|| terms.join(" "))
}
