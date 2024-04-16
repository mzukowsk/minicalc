pub fn error_message(
    expression: &str,
    position: usize,
    underline_length: usize,
    message: &str,
) -> String {
    let expression_length = expression.chars().count();
    let suffix_length = if expression_length <= position + underline_length {
        0
    } else {
        expression_length - underline_length - position
    };
    format!(
        "{}\n{}{}{}\n{}",
        expression,
        " ".repeat(position),
        "^".repeat(underline_length),
        " ".repeat(suffix_length),
        message
    )
}
