#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_span;

use rustc_lint::LateLintPass;

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// Warns when a single-line `return` statement is not preceded by a blank line.
    ///
    /// ### Why is this bad?
    /// Visual separation makes early-exit returns easier to spot when scanning a function.
    pub BLANK_LINE_BEFORE_RETURN,
    Warn,
    "single-line `return` should be preceded by a blank line"
}

impl<'tcx> LateLintPass<'tcx> for BlankLineBeforeReturn {
    // TODO: implement the check + MachineApplicable suggestion that inserts the blank line.
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}
