#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_errors;
extern crate rustc_hir;

use clippy_utils::diagnostics::span_lint_and_then;
use rustc_errors::Applicability;
use rustc_hir::{Block, ImplItemKind, ItemKind, Node, TraitFn, TraitItemKind};
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
        if block.span.from_expansion() {
            return;
        }
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

/// True when `block` is the top-level body of a function — a free `fn`, an
/// inherent or trait-impl method, or a trait method with a default body.
/// False for every other block context (closures, control-flow branches,
/// `let` initializers, nested `{ … }` expressions, …).
///
/// Walk: Block → wrapping `ExprKind::Block` Expr → grandparent. For function
/// bodies the grandparent is the `Fn` item, because `parent_hir_node` skips
/// through the `Body` wrapper; for everything else it's an Expr/Arm/LetStmt.
fn is_fn_body<'tcx>(cx: &LateContext<'tcx>, block: &Block<'_>) -> bool {
    let Node::Expr(parent_expr) = cx.tcx.parent_hir_node(block.hir_id) else {
        return false;
    };
    match cx.tcx.parent_hir_node(parent_expr.hir_id) {
        Node::Item(item) => matches!(item.kind, ItemKind::Fn { .. }),
        Node::ImplItem(item) => matches!(item.kind, ImplItemKind::Fn(..)),
        Node::TraitItem(item) => matches!(item.kind, TraitItemKind::Fn(_, TraitFn::Provided(_))),
        _ => false,
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
