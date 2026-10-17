//! Opt-in native fixtures that build small canisters returning caller-selected raw bytes.

use std::collections::BTreeSet;
use thiserror::Error;
use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, EntityType, ExportKind, ExportSection, Function,
    FunctionSection, ImportSection, Instruction, MemorySection, MemoryType, Module, TypeSection,
    ValType,
};

///
/// ReplyMode
///
/// Explicit IC entry-point mode for one fixture method.
///

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReplyMode {
    /// Export a non-replicated query entry point.
    Query,
    /// Export a replicated update entry point.
    Update,
}

///
/// RawReplyMethod
///
/// One method and its unmodified reply, including malformed Candid when required.
///

#[derive(Clone, Copy, Debug)]
pub struct RawReplyMethod<'a> {
    /// Nonempty method name without control characters; no export prefix.
    pub name: &'a str,
    /// Explicit query or update export.
    pub mode: ReplyMode,
    /// Exact bytes appended before replying; no decoding or normalization.
    pub reply: &'a [u8],
}

///
/// RawReplyLimits
///
/// Caller-selected aggregate admission ceilings checked before module construction.
///

#[derive(Clone, Copy, Debug)]
pub struct RawReplyLimits {
    /// Maximum number of exported methods.
    pub methods: usize,
    /// Maximum sum of all raw reply byte lengths.
    pub reply_bytes: usize,
    /// Maximum sum of all method-name UTF-8 byte lengths, excluding export prefixes.
    pub method_name_bytes: usize,
}

///
/// RawReplyError
///
/// Typed refusal before any simulator, network, or filesystem effects.
///

#[derive(Debug, Eq, Error, PartialEq)]
pub enum RawReplyError {
    /// An aggregate caller ceiling was exceeded, including length overflow.
    #[error("raw reply fixture exceeds {resource} limit {limit}")]
    Budget {
        /// Admission resource name.
        resource: &'static str,
        /// Selected ceiling.
        limit: usize,
    },
    /// Method names must be nonempty and contain no control characters.
    #[error("invalid method name at index {index}")]
    InvalidName {
        /// Input method position.
        index: usize,
    },
    /// Query/update mode and method name must form a unique export.
    #[error("duplicate method export at index {index}")]
    DuplicateExport {
        /// Input method position.
        index: usize,
    },
    /// A function index, reply extent or memory offset exceeds this fixture's Wasm32 model.
    #[error("raw reply fixture cannot represent its lengths in Wasm32")]
    Wasm32Limits,
}

/// Build a canister whose selected query/update methods return exact caller bytes.
///
/// No startup, installation, Candid policy or funding is selected. Reply storage
/// is contiguous and memory cannot grow. The caller owns all method/payload policy.
/// All caller ceilings and Wasm32 extents are checked before encoding the module.
///
/// ```
/// use ic_testkit::pic::fixtures::{
///     raw_reply_canister, RawReplyLimits, RawReplyMethod, ReplyMode,
/// };
/// let wasm = raw_reply_canister(
///     &[RawReplyMethod { name: "probe", mode: ReplyMode::Query, reply: b"raw" }],
///     RawReplyLimits { methods: 1, reply_bytes: 3, method_name_bytes: 5 },
/// )?;
/// assert!(wasm.starts_with(b"\0asm"));
/// # Ok::<(), ic_testkit::pic::fixtures::RawReplyError>(())
/// ```
///
/// # Errors
/// Returns a typed error for exceeded limits, invalid/duplicate names or unsupported extents.
pub fn raw_reply_canister(
    methods: &[RawReplyMethod<'_>],
    limits: RawReplyLimits,
) -> Result<Vec<u8>, RawReplyError> {
    if methods.len() > limits.methods {
        return Err(RawReplyError::Budget {
            resource: "methods",
            limit: limits.methods,
        });
    }
    let count = u32::try_from(methods.len()).map_err(|_| RawReplyError::Wasm32Limits)?;
    count.checked_add(2).ok_or(RawReplyError::Wasm32Limits)?;
    let mut exports = BTreeSet::new();
    let mut bytes = 0_usize;
    let mut names = 0_usize;
    for (index, method) in methods.iter().enumerate() {
        if method.name.is_empty() || method.name.chars().any(char::is_control) {
            return Err(RawReplyError::InvalidName { index });
        }
        if !exports.insert((method.mode, method.name)) {
            return Err(RawReplyError::DuplicateExport { index });
        }
        names = charge(
            names,
            method.name.len(),
            limits.method_name_bytes,
            "method names",
        )?;
        bytes = charge(bytes, method.reply.len(), limits.reply_bytes, "reply bytes")?;
    }
    i32::try_from(bytes).map_err(|_| RawReplyError::Wasm32Limits)?;
    let pages =
        u64::try_from(bytes.div_ceil(65_536).max(1)).map_err(|_| RawReplyError::Wasm32Limits)?;
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32, ValType::I32], []);
    types.ty().function([], []);
    module.section(&types);
    let mut imports = ImportSection::new();
    imports.import("ic0", "msg_reply_data_append", EntityType::Function(0));
    imports.import("ic0", "msg_reply", EntityType::Function(1));
    module.section(&imports);
    let mut functions = FunctionSection::new();
    for _ in methods {
        functions.function(1);
    }
    module.section(&functions);
    let mut memory = MemorySection::new();
    memory.memory(MemoryType {
        minimum: pages,
        maximum: Some(pages),
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memory);
    let mut exported = ExportSection::new();
    exported.export("memory", ExportKind::Memory, 0);
    let mut code = CodeSection::new();
    let mut data = DataSection::new();
    let mut offset = 0_i32;
    for (index, method) in (2..count + 2).zip(methods) {
        let prefix = match method.mode {
            ReplyMode::Query => "canister_query",
            ReplyMode::Update => "canister_update",
        };
        exported.export(
            &format!("{prefix} {}", method.name),
            ExportKind::Func,
            index,
        );
        let length = i32::try_from(method.reply.len()).map_err(|_| RawReplyError::Wasm32Limits)?;
        let mut body = Function::new([]);
        body.instruction(&Instruction::I32Const(offset));
        body.instruction(&Instruction::I32Const(length));
        body.instruction(&Instruction::Call(0));
        body.instruction(&Instruction::Call(1));
        body.instruction(&Instruction::End);
        code.function(&body);
        data.active(
            0,
            &ConstExpr::i32_const(offset),
            method.reply.iter().copied(),
        );
        offset += length;
    }
    module.section(&exported);
    module.section(&code);
    module.section(&data);
    Ok(module.finish())
}

fn charge(
    total: usize,
    amount: usize,
    limit: usize,
    resource: &'static str,
) -> Result<usize, RawReplyError> {
    total
        .checked_add(amount)
        .filter(|sum| *sum <= limit)
        .ok_or(RawReplyError::Budget { resource, limit })
}

#[cfg(test)]
mod tests;
