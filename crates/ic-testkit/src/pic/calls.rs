use candid::{CandidType, Principal, decode_one, encode_args, utils::ArgumentEncoder};
use pocket_ic::{PocketIc, RejectResponse};
use serde::de::DeserializeOwned;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

use super::{CandidCallContext, CandidCallError, transport};

#[derive(Clone, Copy)]
struct CallContext<'a> {
    operation: &'static str,
    canister_id: Principal,
    caller: Principal,
    method: &'a str,
}

impl CallContext<'_> {
    fn to_error_context(self) -> CandidCallContext {
        CandidCallContext::new(self.operation, self.canister_id, self.caller, self.method)
    }
}

/// Typed Candid calls with contextual encoding, rejection, and decoding errors.
pub trait CandidCallExt {
    /// Encode and execute an anonymous update call, then decode its result.
    fn update_candid<T, A>(
        &self,
        canister_id: Principal,
        method: &str,
        args: A,
    ) -> Result<T, CandidCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder;

    /// Execute [`update_candid`](Self::update_candid), panicking on harness errors.
    #[track_caller]
    fn update_candid_or_panic<T, A>(&self, canister_id: Principal, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder;

    /// Encode and execute an update call as an explicit caller.
    fn update_candid_as<T, A>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: A,
    ) -> Result<T, CandidCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder;

    /// Execute [`update_candid_as`](Self::update_candid_as), panicking on harness errors.
    #[track_caller]
    fn update_candid_as_or_panic<T, A>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: A,
    ) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder;

    /// Encode and execute an anonymous query call, then decode its result.
    fn query_candid<T, A>(
        &self,
        canister_id: Principal,
        method: &str,
        args: A,
    ) -> Result<T, CandidCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder;

    /// Execute [`query_candid`](Self::query_candid), panicking on harness errors.
    #[track_caller]
    fn query_candid_or_panic<T, A>(&self, canister_id: Principal, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder;

    /// Encode and execute a query call as an explicit caller.
    fn query_candid_as<T, A>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: A,
    ) -> Result<T, CandidCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder;

    /// Execute [`query_candid_as`](Self::query_candid_as), panicking on harness errors.
    #[track_caller]
    fn query_candid_as_or_panic<T, A>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: A,
    ) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder;
}

impl CandidCallExt for PocketIc {
    /// Generic update call helper (serializes args + decodes result).
    fn update_candid<T, A>(
        &self,
        canister_id: Principal,
        method: &str,
        args: A,
    ) -> Result<T, CandidCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.update_candid_as(canister_id, Principal::anonymous(), method, args)
    }

    /// Generic update call helper that panics on rejection or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `update_candid_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    fn update_candid_or_panic<T, A>(&self, canister_id: Principal, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.update_candid(canister_id, method, args)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Generic update call helper with an explicit caller principal.
    fn update_candid_as<T, A>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: A,
    ) -> Result<T, CandidCallError>
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
        let result = run_raw_call(context, || {
            Self::update_call(self, canister_id, caller, method, bytes)
        })?;

        decode_call_result(&result, context)
    }

    /// Generic update call helper with an explicit caller principal that panics
    /// on rejection or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `update_candid_as_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    fn update_candid_as_or_panic<T, A>(
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
        self.update_candid_as(canister_id, caller, method, args)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Generic query call helper.
    fn query_candid<T, A>(
        &self,
        canister_id: Principal,
        method: &str,
        args: A,
    ) -> Result<T, CandidCallError>
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.query_candid_as(canister_id, Principal::anonymous(), method, args)
    }

    /// Generic query call helper that panics on rejection or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `query_candid_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    fn query_candid_or_panic<T, A>(&self, canister_id: Principal, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.query_candid(canister_id, method, args)
            .unwrap_or_else(|err| panic!("{err}"))
    }

    /// Generic query call helper with an explicit caller principal.
    fn query_candid_as<T, A>(
        &self,
        canister_id: Principal,
        caller: Principal,
        method: &str,
        args: A,
    ) -> Result<T, CandidCallError>
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
        let result = run_raw_call(context, || {
            Self::query_call(self, canister_id, caller, method, bytes)
        })?;

        decode_call_result(&result, context)
    }

    /// Generic query call helper with an explicit caller principal that panics
    /// on rejection or Candid codec failure.
    ///
    /// This does not unwrap application-level results. For example,
    /// `query_candid_as_or_panic::<Result<T, E>, _>(...)` returns `Result<T, E>`.
    #[track_caller]
    fn query_candid_as_or_panic<T, A>(
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
        self.query_candid_as(canister_id, caller, method, args)
            .unwrap_or_else(|err| panic!("{err}"))
    }
}

fn encode_call_args<A>(args: A, context: CallContext<'_>) -> Result<Vec<u8>, CandidCallError>
where
    A: ArgumentEncoder,
{
    encode_args(args).map_err(|err| CandidCallError::encode(context.to_error_context(), err))
}

fn decode_call_result<T>(result: &[u8], context: CallContext<'_>) -> Result<T, CandidCallError>
where
    T: CandidType + DeserializeOwned,
{
    decode_one(result)
        .map_err(|err| CandidCallError::decode(context.to_error_context(), result.len(), err))
}

fn run_raw_call<F>(context: CallContext<'_>, call: F) -> Result<Vec<u8>, CandidCallError>
where
    F: FnOnce() -> Result<Vec<u8>, RejectResponse>,
{
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(response)) => Err(CandidCallError::canister_reject(
            context.to_error_context(),
            response,
        )),
        Err(payload) if transport::panic_is_dead_instance_transport(payload.as_ref()) => {
            Err(CandidCallError::transport(
                context.to_error_context(),
                transport::panic_payload_to_string(payload.as_ref()),
            ))
        }
        Err(payload) => resume_unwind(payload),
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;

    use crate::pic::CandidCallErrorKind;

    use super::{CallContext, decode_call_result, run_raw_call};

    #[test]
    fn decode_error_includes_call_context() {
        let context = CallContext {
            operation: "query_call",
            canister_id: Principal::anonymous(),
            caller: Principal::management_canister(),
            method: "get",
        };

        let err = decode_call_result::<u64>(&[0xde, 0xad], context).expect_err("decode fails");

        assert!(err.message().contains("candid decode_one failed"));
        assert!(err.message().contains("operation=query_call"));
        assert!(err.message().contains("method=get"));
        assert!(err.message().contains("bytes=2"));
        assert_eq!(err.kind(), CandidCallErrorKind::Decode);
        assert_eq!(err.context().expect("decode error context").method(), "get");
    }

    #[test]
    fn dead_instance_panic_is_classified_as_transport() {
        let context = CallContext {
            operation: "query_call",
            canister_id: Principal::anonymous(),
            caller: Principal::management_canister(),
            method: "get",
        };
        let error = run_raw_call(context, || -> Result<Vec<u8>, pocket_ic::RejectResponse> {
            panic!("transport failed: ConnectionRefused");
        })
        .unwrap_err();

        assert_eq!(error.kind(), crate::pic::CandidCallErrorKind::Transport);
        assert!(error.reject_response().is_none());
    }
}
