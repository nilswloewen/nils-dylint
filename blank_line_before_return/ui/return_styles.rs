// Showcase of different return styles. Inline `// WARN` / `// ok` comments
// document the expected behaviour per case; the lint only flags single-line
// implicit tail returns that lack a blank line above them.

#![allow(dead_code, unused_variables)]

// 1. Plain identifier tail after a `let`.
fn implicit_ident() -> i32 {
    let x = 1;
    x // WARN
}

// 2. Same shape, but with the conventional blank line.
fn implicit_with_blank() -> i32 {
    let x = 1;

    x // ok
}

// 3. Function whose only line is the tail — no previous statement to be
//    separated from.
fn body_just_tail() -> i32 {
    42 // ok
}

// 4. Explicit `return x;` — outside the lint's scope.
fn explicit_only() -> i32 {
    let x = compute();
    return x;
}

// 5. Early explicit `return` followed by an implicit tail. The lint targets
//    only the trailing implicit return, which here is missing its blank line.
fn explicit_early_then_implicit() -> i32 {
    let x = compute();
    if x < 0 {
        return 0;
    }
    x // WARN
}

// 6. Single-line `if / else` expression as the tail.
fn implicit_if_else() -> i32 {
    let x = compute();
    if x > 0 { 1 } else { -1 } // WARN
}

// 7. Multi-line `match` as the tail — exempt because it spans multiple lines.
fn implicit_match() -> i32 {
    let x = compute();
    match x {
        0 => 0,
        _ => 1,
    } // ok
}

// 8. Method-chain tail.
fn implicit_method_chain() -> String {
    let s = String::from("hi");
    s.to_uppercase() // WARN
}

// 9. `?` followed by a single-line `Ok(...)` tail.
fn implicit_try() -> Result<usize, std::num::ParseIntError> {
    let n: i32 = "42".parse()?;
    Ok(n as usize) // WARN
}

// 10. Closure body — also a block, also checked.
fn closure_tail() {
    let _f = || {
        let x = 1;
        x // WARN
    };
}

// 11. Nested blocks. The inner block already has a blank line; the outer
//     block's tail (the inner block) is multi-line, so neither warns.
fn nested_blocks() -> i32 {
    let n = 10;
    {
        let m = 5;

        m + n // ok
    } // ok
}

// 12. Unit tail expression after a non-unit statement.
fn implicit_unit() {
    let _ = compute();
    () // WARN
}

// 13. Tail after a macro-call statement — `source_callsite()` ensures the
//     macro stmt is still treated as the previous source line.
fn after_macro_stmt() -> i32 {
    println!("computing…");
    compute() // WARN
}

// 14. Closure as the tail expression of a block — exempt. Factory-style
//     "build a closure" blocks shouldn't be visually re-flowed.
fn returns_closure() -> impl Fn() -> i32 {
    let n = compute();
    move || n // ok — closure tail
}

// 15. Inside match-arm bodies the lint is silenced, even when the arm's tail
//     has no blank line above it — matches the user's preferred style for
//     dense arm lists.
fn match_arm_bodies() -> Result<i32, &'static str> {
    let n = compute();
    match n {
        0 => {
            let _shadow = n;
            Ok(0) // ok — inside a match arm
        }
        _ => match n.signum() {
            1 => {
                let _shadow = n;
                Ok(n) // ok — inside a nested match arm
            }
            _ => Err("negative"),
        },
    }
}

fn compute() -> i32 {
    0
}

fn main() {}
