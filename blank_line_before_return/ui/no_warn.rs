// None of these should trigger BLANK_LINE_BEFORE_RETURN.

// Already has a blank line above the tail.
fn has_blank_line() -> i32 {
    let x = 1;

    x
}

// Body is just the tail expression — nothing to be separated from.
fn only_tail() -> i32 {
    42
}

// Explicit `return x;` is out of scope; the lint targets implicit tail returns only.
fn explicit_return() -> i32 {
    let x = 1;
    return x;
}

// Multi-line tail expressions are exempt.
fn multi_line_tail() -> Vec<i32> {
    let x = 1;
    vec![
        x,
        x + 1,
        x + 2,
    ]
}

// A comment line between the previous statement and the tail acts as visual separation.
fn comment_before_tail() -> i32 {
    let x = 1;
    // returning x
    x
}

// Empty block — nothing to check.
fn returns_unit() {
    let _ = 1;
}

fn main() {}
