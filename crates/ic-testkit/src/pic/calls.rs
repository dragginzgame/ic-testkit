use candid::{CandidType, Principal, decode_one, encode_args, utils::ArgumentEncoder};
use serde::de::DeserializeOwned;

use super::{Pic, PicCallError};

#[derive(Clone, Copy)]
struct CallContext<'a> {
    operation: &'static str,
    canister_id: Principal,
    caller: Principal,
    method: &'a str,
}

impl Pic {
    /// Generic update call helper (serializes args + decodes result).
    pub fn update_call<T, A>(
        &self,
        canister_id: Principal,
        method: &str,
        args: A,
    ) -> Result<T, PicCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.update_call_as(canister_id, Principal::anonymous(), method, args)
    }

    /// Generic update call helper that panics on transport or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `update_call_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    pub fn update_call_or_panic<T, A>(&self, canister_id: Principal, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.update_call(canister_id, method, args)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Generic update call helper with an explicit caller principal.
    pub fn update_call_as<T, A>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: A,
    ) -> Result<T, PicCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        let context = CallContext {
            operation: "update_call",
            canister_id,
            caller,
            method,
        };
        let bytes = encode_call_args(args, context)?;
        let result = self
            .inner
            .update_call(canister_id, caller, method, bytes)
            .map_err(|err| {
                PicCallError::new(format!(
                    "pocket_ic update_call failed (canister={canister_id}, caller={caller}, method={method}): {err}"
                ))
            })?;

        decode_call_result(&result, context)
    }

    /// Generic update call helper with an explicit caller principal that panics
    /// on transport or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `update_call_as_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    pub fn update_call_as_or_panic<T, A>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: A,
    ) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.update_call_as(canister_id, caller, method, args)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Generic query call helper.
    pub fn query_call<T, A>(
        &self,
        canister_id: Principal,
        method: &str,
        args: A,
    ) -> Result<T, PicCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.query_call_as(canister_id, Principal::anonymous(), method, args)
    }

    /// Generic query call helper that panics on transport or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `query_call_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    pub fn query_call_or_panic<T, A>(&self, canister_id: Principal, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.query_call(canister_id, method, args)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Generic query call helper with an explicit caller principal.
    pub fn query_call_as<T, A>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: A,
    ) -> Result<T, PicCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        let context = CallContext {
            operation: "query_call",
            canister_id,
            caller,
            method,
        };
        let bytes = encode_call_args(args, context)?;
        let result = self
            .inner
            .query_call(canister_id, caller, method, bytes)
            .map_err(|err| {
                PicCallError::new(format!(
                    "pocket_ic query_call failed (canister={canister_id}, caller={caller}, method={method}): {err}"
                ))
            })?;

        decode_call_result(&result, context)
    }

    /// Generic query call helper with an explicit caller principal that panics
    /// on transport or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `query_call_as_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    pub fn query_call_as_or_panic<T, A>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: A,
    ) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.query_call_as(canister_id, caller, method, args)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Advance PocketIC by a fixed number of ticks.
    pub fn tick_n(&self, times: usize) {
        for _ in 0..times {
            self.tick();
        }
    }
}

fn encode_call_args<A>(args: A, context: CallContext<'_>) -> Result<Vec<u8>, PicCallError>
where
    A: ArgumentEncoder,
{
    encode_args(args).map_err(|err| {
        PicCallError::new(format!(
            "candid encode_args failed (operation={}, canister={}, caller={}, method={}): {err}",
            context.operation, context.canister_id, context.caller, context.method
        ))
    })
}

fn decode_call_result<T>(result: &[u8], context: CallContext<'_>) -> Result<T, PicCallError>
where
    T: CandidType + DeserializeOwned,
{
    decode_one(result).map_err(|err| {
        PicCallError::new(format!(
            "candid decode_one failed (operation={}, canister={}, caller={}, method={}, bytes={}): {err}",
            context.operation,
            context.canister_id,
            context.caller,
            context.method,
            result.len()
        ))
    })
}

#[cfg(test)]
mod tests {
    use candid::Principal;

    use super::{CallContext, decode_call_result};

    #[test]
    fn decode_error_includes_call_context() {
        let context = CallContext {
            operation: "query_call",
            canister_id: Principal::anonymous(),
            caller: Principal::management_canister(),
            method: "get",
        };

        let err = decode_call_result::<u64>(&[0xde, 0xad], context).expect_err("decode fails");

        assert!(err.message.contains("candid decode_one failed"));
        assert!(err.message.contains("operation=query_call"));
        assert!(err.message.contains("method=get"));
        assert!(err.message.contains("bytes=2"));
    }
}
