#![no_std]

#[expect(
    mismatched_lifetime_syntaxes,
    non_snake_case,
    non_camel_case_types,
    unused_imports,
    clippy::missing_safety_doc,
    clippy::extra_unused_lifetimes,
    clippy::derivable_impls,
    clippy::doc_lazy_continuation
)]
#[path = "../target/flatbuffers/Message_generated.rs"]
mod generated;
pub use generated::*;
