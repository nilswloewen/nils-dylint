// Every function in this file has a single-line tail-return expression that is
// NOT preceded by a blank line — each should trigger BLANK_LINE_BEFORE_RETURN.

fn after_let() -> i32 {
    let x = 1;
    x
}

fn after_expr_stmt() -> i32 {
    println!("hi");
    42
}

fn after_block() -> i32 {
    if std::env::var("X").is_ok() {
        println!("set");
    }
    7
}

fn after_explicit_return_branch() -> i32 {
    if false {
        return 1;
    }
    2
}

fn tuple_tail() -> (i32, i32) {
    let a = 1;
    (a, a + 1)
}

fn nested_block_tail() -> i32 {
    let outer = 10;
    {
        let inner = 1;
        inner + outer
    }
}

fn main() {}
