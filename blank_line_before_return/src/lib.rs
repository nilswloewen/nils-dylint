#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_errors;
extern crate rustc_hir;

use clippy_utils::diagnostics::span_lint_and_then;
use rustc_errors::Applicability;
use rustc_hir::{
    Block, ClosureKind, Expr, ExprKind, ImplItemKind, ItemKind, Node, TraitFn, TraitItemKind,
};
use rustc_lint::{LateContext, LateLintPass, LintContext};

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// Warns when a **function body**'s implicit tail-return expression fits on a
    /// single line and is not preceded by a blank line.
    ///
    /// Only function bodies are checked — closure bodies, `if`/`else` branches,
    /// `match` arms, `let`-initializer blocks (`let x = { … };`), and any other
    /// inner block expressions are left alone. Explicit `return x;` is also out
    /// of scope.
    ///
    /// ### Why is this bad?
    /// A blank line above the tail expression makes the function's return value
    /// easier to spot when scanning. The same padding inside inner blocks reads
    /// as noise — the enclosing construct (`|args|`, `if`, `match { … }`, the
    /// `let` binding) already brackets the code visually.
    ///
    /// ### Example
    /// ```rust
    /// fn foo() -> i32 {
    ///     let x = compute();
    ///     x
    /// }
    /// ```
    /// Use instead:
    /// ```rust
    /// fn foo() -> i32 {
    ///     let x = compute();
    ///
    ///     x
    /// }
    /// ```
    pub BLANK_LINE_BEFORE_RETURN,
    Warn,
    "function-body tail return should be preceded by a blank line"
}

impl<'tcx> LateLintPass<'tcx> for BlankLineBeforeReturn {
    fn check_block(&mut self, cx: &LateContext<'tcx>, block: &'tcx Block<'tcx>) {
        let Some(tail) = block.expr else { return };
        let Some(prev) = block.stmts.last() else { return };

        if !is_fn_body(cx, block) {
            return;
        }

        // `source_callsite()` walks out of any macro expansion to the user-visible
        // call site, so macro-call statements (`println!(…);`) and macro-produced
        // tail expressions are measured by where they appear in the source.
        let tail_span = tail.span.source_callsite();
        let prev_span = prev.span.source_callsite();

        let src_map = cx.sess().source_map();
        let tail_lo = src_map.lookup_char_pos(tail_span.lo());
        let tail_hi = src_map.lookup_char_pos(tail_span.hi());
        let prev_hi = src_map.lookup_char_pos(prev_span.hi());

        // Only flag single-line tail expressions.
        if tail_lo.line != tail_hi.line {
            return;
        }

        // Skip if there's already a gap (blank line, comment line, …) between the
        // previous statement and the tail, or if they share a line.
        if tail_lo.line != prev_hi.line + 1 {
            return;
        }

        // Zero-width span at column 0 of the tail's line, so inserting "\n" there adds
        // a blank line above the existing indentation rather than after it.
        // `Loc::line` is 1-indexed; `SourceFile::line_bounds` is 0-indexed.
        let line_start = tail_lo.file.line_bounds(tail_lo.line - 1).start;
        let insert_span = tail_span.with_lo(line_start).shrink_to_lo();

        span_lint_and_then(
            cx,
            BLANK_LINE_BEFORE_RETURN,
            tail_span,
            "missing blank line before the trailing return expression",
            |diag| {
                diag.span_suggestion(
                    insert_span,
                    "insert a blank line",
                    "\n",
                    Applicability::MachineApplicable,
                );
            },
        );
    }
}

/// True when `block` is the body the user wrote for a function — a free `fn`, an
/// inherent or trait-impl method, or a trait method with a default body.
/// False for every other block context (closures, control-flow branches,
/// `let` initializers, nested `{ … }` expressions, …).
///
/// Walks from `block` up to the enclosing item, stepping over nodes that the
/// user did not write. A block only counts as a function body if *everything*
/// between it and the `fn` is scaffolding:
///
/// * `fn foo() { … }` — the block sits directly under the item.
/// * `async fn foo() { … }` — the body is lowered to a coroutine, burying the
///   block under `DropTemps` and an extra block: `Item → Closure → Block →
///   DropTemps → block`.
/// * `#[tracing::instrument] async fn foo() { … }` — the attribute macro moves
///   the body into a generated `let fut = async move { … };`, so the block is
///   additionally wrapped in macro-generated nodes.
///
/// Any node the user *did* write (an `if`, a `match` arm, a real `let`, a
/// hand-written closure or `async` block) stops the walk and yields false.
fn is_fn_body<'tcx>(cx: &LateContext<'tcx>, block: &Block<'_>) -> bool {
    let mut hir_id = block.hir_id;

    // Set once the walk steps through macro-generated code, which relaxes the
    // rule for enclosing blocks below. An attribute macro that moves the body
    // (`#[tracing::instrument]`) injects its scaffolding *inside* the `fn`'s
    // original braces, so the block it lands in still has a real span even
    // though the user did not write that nesting.
    let mut crossed_expansion = false;

    loop {
        match cx.tcx.parent_hir_node(hir_id) {
            Node::Item(item) => return matches!(item.kind, ItemKind::Fn { .. }),
            Node::ImplItem(item) => return matches!(item.kind, ImplItemKind::Fn(..)),
            Node::TraitItem(item) => {
                return matches!(item.kind, TraitItemKind::Fn(_, TraitFn::Provided(_)));
            },
            Node::Expr(expr) => {
                if !is_scaffolding(expr) {
                    return false;
                }
                crossed_expansion |= expr.span.from_expansion();
                hir_id = expr.hir_id;
            },
            // Only climb out of a block the user wrote once inside an expansion;
            // otherwise this is a plain inner `{ … }` block and stays skipped.
            Node::Block(outer) => {
                if !outer.span.from_expansion() && !crossed_expansion {
                    return false;
                }
                hir_id = outer.hir_id;
            },
            Node::Stmt(stmt) => {
                if !stmt.span.from_expansion() && !crossed_expansion {
                    return false;
                }
                hir_id = stmt.hir_id;
            },
            // A real `let` means the block is an initializer the user wrote.
            Node::LetStmt(let_stmt) => {
                if !let_stmt.span.from_expansion() {
                    return false;
                }
                crossed_expansion = true;
                hir_id = let_stmt.hir_id;
            },
            _ => return false,
        }
    }
}

/// True for a node between a function body block and its `fn` that the user did
/// not write: the `ExprKind::Block` wrapper every block carries, the `DropTemps`
/// and coroutine layers of the `async fn` desugaring, and anything a macro
/// generated.
fn is_scaffolding(expr: &Expr<'_>) -> bool {
    match expr.kind {
        // The wrapper expression a block is always parented by.
        ExprKind::Block(..) | ExprKind::DropTemps(..) => true,
        // The coroutine an `async fn` body is lowered into. `is_fn_like` matches
        // only that desugaring — a hand-written `async { … }` block or async
        // closure is `CoroutineSource::Block`/`Closure` and is not scaffolding
        // unless a macro generated it.
        ExprKind::Closure(closure) => match closure.kind {
            ClosureKind::Coroutine(kind) => kind.is_fn_like() || expr.span.from_expansion(),
            _ => expr.span.from_expansion(),
        },
        _ => expr.span.from_expansion(),
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
