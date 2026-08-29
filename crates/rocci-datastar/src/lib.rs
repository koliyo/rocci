pub mod assets;
pub mod codegen;
pub mod error;
pub mod signals;
pub mod spec;
pub mod sse;

pub use assets::{DEFAULT_VERSION, parse_version, tag_name};
pub use codegen::{DATASTAR_ROC_TEMPLATE, stage_datastar_roc};
pub use error::{DatastarError, Result};
pub use signals::{from_json_str, from_query_str};
pub use spec::{
    ACTIONS, ATTRIBUTES, ActionSpec, AttributeSpec, MODIFIERS, ModifierSpec, ParsedAttribute,
    ParsedModifier, is_datastar_attribute, lookup_action, lookup_attribute, lookup_modifier,
    parse_attribute,
};
pub use sse::{
    ExecuteScript, PatchElements, PatchMode, PatchSignals, RemoveFragments, strip_style_elements,
};
