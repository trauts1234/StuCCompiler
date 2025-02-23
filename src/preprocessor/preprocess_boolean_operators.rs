
pub fn get_binary_numerical_text_and_functions<'a>() -> Vec<(&'a str, fn(i64, i64) -> i64)> {
    get_binary_numerical_operator_text().into_iter()
    .zip(get_binary_numerical_operator_function().into_iter())
    .collect()
}

pub fn get_relational_text_and_functions<'a>() -> Vec<(&'a str, fn(i64, i64) -> i64)> {
    todo!("borrow from binary_numerical, but for >, <, >=, <= as per order of operations")
}

fn get_binary_numerical_operator_text<'a>() -> Vec<&'a str> {
    vec!["||", "&&", "|", "^", "&"]
}

fn get_binary_numerical_operator_function() -> Vec<fn(i64, i64) -> i64> {
    vec![
        |x, y| if (x != 0) || (y != 0) { 1 } else { 0 },
        |x, y| if (x != 0) && (y != 0) { 1 } else { 0 },
        |x, y| x | y,
        |x, y| x ^ y,
        |x, y| x & y,
    ]
}