// Creusot-facing representation used only while verifying the requirement
// evaluator. The published pointer-tagged implementation remains compiled in
// every ordinary build. Evaluation observes identifiers exclusively through
// the reviewed empty/comparison contracts in `eval.rs`.
pub(crate) struct Identifier {
    empty: bool,
}

impl Identifier {
    pub(crate) fn is_empty(&self) -> bool {
        self.empty
    }
}
