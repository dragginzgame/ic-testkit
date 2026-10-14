use candid::{
    CandidType, Principal, decode_args, decode_one, encode_args,
    utils::{ArgumentDecoder, ArgumentEncoder},
};
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
        A: ArgumentEncoder,
    {
        self.update_candid_as(canister_id, Principal::anonymous(), method, args)
    }

    /// Execute [`update_candid`](Self::update_candid), panicking on harness errors.
    ///
    /// Application-level `Result<T, E>` values are returned without unwrapping.
    #[track_caller]
    fn update_candid_or_panic<T, A>(&self, canister_id: Principal, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.update_candid(canister_id, method, args)
            .unwrap_or_else(|err| panic!("{err}"))
    }

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
    ///
    /// Application-level `Result<T, E>` values are returned without unwrapping.
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

    /// Encode and execute an anonymous query call, then decode its result.
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

    /// Execute [`query_candid`](Self::query_candid), panicking on harness errors.
    ///
    /// Application-level `Result<T, E>` values are returned without unwrapping.
    #[track_caller]
    fn query_candid_or_panic<T, A>(&self, canister_id: Principal, method: &str, args: A) -> T
    where
        T: CandidType + DeserializeOwned,
        A: ArgumentEncoder,
    {
        self.query_candid(canister_id, method, args)
            .unwrap_or_else(|err| panic!("{err}"))
    }

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
    ///
    /// Application-level `Result<T, E>` values are returned without unwrapping.
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

impl CandidCallExt for PocketIc {
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
        call_candid(
            context,
            args,
            |bytes| Self::update_call(self, canister_id, caller, method, bytes),
            |bytes| decode_one(bytes),
        )
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
        call_candid(
            context,
            args,
            |bytes| Self::query_call(self, canister_id, caller, method, bytes),
            |bytes| decode_one(bytes),
        )
    }
}

/// Execute an update as `caller`, decoding the top-level Candid reply arguments.
///
/// Unlike [`CandidCallExt::update_candid_as`], `(u64, bool)` means two reply
/// arguments rather than one tuple-valued reply. Use `()` for no replies or
/// `(T,)` for one reply. Pass [`Principal::anonymous()`] for an anonymous caller.
/// Application-level `Result<T, E>` values are returned without unwrapping.
pub fn update_candid_args<T, A>(
    pic: &PocketIc,
    canister_id: Principal,
    caller: Principal,
    method: &str,
    args: A,
) -> Result<T, CandidCallError>
where
    T: for<'de> ArgumentDecoder<'de>,
    A: ArgumentEncoder,
{
    let context = CallContext {
        operation: "update_call",
        canister_id,
        caller,
        method,
    };
    call_candid(
        context,
        args,
        |bytes| pic.update_call(canister_id, caller, method, bytes),
        |bytes| decode_args(bytes),
    )
}

/// Execute a query as `caller`, decoding the top-level Candid reply arguments.
///
/// Unlike [`CandidCallExt::query_candid_as`], `(u64, bool)` means two reply
/// arguments rather than one tuple-valued reply. Use `()` for no replies or
/// `(T,)` for one reply. Pass [`Principal::anonymous()`] for an anonymous caller.
/// Application-level `Result<T, E>` values are returned without unwrapping.
pub fn query_candid_args<T, A>(
    pic: &PocketIc,
    canister_id: Principal,
    caller: Principal,
    method: &str,
    args: A,
) -> Result<T, CandidCallError>
where
    T: for<'de> ArgumentDecoder<'de>,
    A: ArgumentEncoder,
{
    let context = CallContext {
        operation: "query_call",
        canister_id,
        caller,
        method,
    };
    call_candid(
        context,
        args,
        |bytes| pic.query_call(canister_id, caller, method, bytes),
        |bytes| decode_args(bytes),
    )
}

fn call_candid<T, A, F, D>(
    context: CallContext<'_>,
    args: A,
    call: F,
    decode: D,
) -> Result<T, CandidCallError>
where
    A: ArgumentEncoder,
    F: FnOnce(Vec<u8>) -> Result<Vec<u8>, RejectResponse>,
    D: FnOnce(&[u8]) -> Result<T, candid::Error>,
{
    let bytes = encode_args(args)
        .map_err(|err| CandidCallError::encode(context.to_error_context(), err))?;
    let result = run_raw_call(context, || call(bytes))?;
    decode(&result)
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

    use super::{CallContext, call_candid, run_raw_call};

    #[test]
    fn decode_error_includes_call_context() {
        let context = CallContext {
            operation: "query_call",
            canister_id: Principal::anonymous(),
            caller: Principal::management_canister(),
            method: "get",
        };

        let err = call_candid(
            context,
            (),
            |_| Ok(vec![0xde, 0xad]),
            |bytes| candid::decode_one::<u64>(bytes),
        )
        .expect_err("decode fails");

        assert!(err.message().contains("candid decode failed"));
        assert!(err.message().contains("operation=query_call"));
        assert!(err.message().contains("method=get"));
        assert!(err.message().contains("bytes=2"));
        assert_eq!(err.kind(), CandidCallErrorKind::Decode);
        assert_eq!(err.context().expect("decode error context").method(), "get");
    }

    #[test]
    fn reply_arguments_preserve_top_level_arity_and_wide_values() {
        let context = CallContext {
            operation: "update_call",
            canister_id: Principal::anonymous(),
            caller: Principal::management_canister(),
            method: "measure",
        };
        let values = (u128::MAX, u128::from(u64::MAX) + 1);
        let bytes = candid::encode_args(values).unwrap();
        assert_eq!(candid::decode_args::<(u128, u128)>(&bytes).unwrap(), values);
        assert!(candid::decode_one::<(u128, u128)>(&bytes).is_err());
        let tuple = candid::encode_one(values).unwrap();
        assert!(candid::decode_args::<(u128, u128)>(&tuple).is_err());
        assert_eq!(candid::decode_one::<(u128, u128)>(&tuple).unwrap(), values);
        let no_replies = candid::encode_args(()).unwrap();
        candid::decode_args::<()>(&no_replies).unwrap();
        let one_reply = candid::encode_args((7_u64,)).unwrap();
        assert_eq!(candid::decode_args::<(u64,)>(&one_reply).unwrap(), (7,));
        let result: (u64, bool, String) = call_candid(
            context,
            (42_u64, false),
            |args| {
                assert_eq!(
                    candid::decode_args::<(u64, bool)>(&args).unwrap(),
                    (42, false)
                );
                Ok(candid::encode_args((9_u64, true, "ready")).unwrap())
            },
            |bytes| candid::decode_args(bytes),
        )
        .unwrap();
        assert_eq!(result, (9, true, "ready".into()));
    }

    #[test]
    fn invalid_reply_arguments_keep_decode_context() {
        let context = CallContext {
            operation: "query_call",
            canister_id: Principal::anonymous(),
            caller: Principal::management_canister(),
            method: "measure",
        };
        for bytes in [vec![0xde, 0xad], candid::encode_args((7_u64,)).unwrap()] {
            let length = bytes.len();
            let err = call_candid(
                context,
                (),
                |_| Ok(bytes),
                |bytes| candid::decode_args::<(u64, bool)>(bytes),
            )
            .unwrap_err();
            assert_eq!(err.kind(), CandidCallErrorKind::Decode);
            let observed = err.context().unwrap();
            assert_eq!(observed.operation(), "query_call");
            assert_eq!(observed.canister_id(), context.canister_id);
            assert_eq!(observed.caller(), context.caller);
            assert_eq!(observed.method(), "measure");
            assert!(err.message().contains(&format!("bytes={length}")));
        }
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
            panic!("HTTP failure: reqwest::Error {{ kind: Request, url: \"http://127.0.0.1:1234/instances/0/update/execute_ingress_message\", source: ConnectError(\"tcp connect error\", Os {{ code: 111, kind: ConnectionRefused, message: \"Connection refused\" }}) }}");
        })
        .unwrap_err();

        assert_eq!(error.kind(), crate::pic::CandidCallErrorKind::Transport);
        assert!(error.reject_response().is_none());
    }

    #[test]
    fn unrelated_panics_resume_with_the_original_payload() {
        for message in [
            "application worker channel closed",
            "fixture expected ConnectionRefused but observed another result",
        ] {
            let context = CallContext {
                operation: "query_call",
                canister_id: Principal::anonymous(),
                caller: Principal::management_canister(),
                method: "get",
            };
            let payload = message.to_owned();
            let original_allocation = payload.as_ptr() as usize;
            let result = std::panic::catch_unwind(|| {
                run_raw_call(context, || -> Result<Vec<u8>, pocket_ic::RejectResponse> {
                    std::panic::panic_any(payload);
                })
            });
            let resumed = result
                .expect_err("application panic must resume")
                .downcast::<String>()
                .expect("original panic type must be preserved");
            assert_eq!(resumed.as_str(), message);
            assert_eq!(resumed.as_ptr() as usize, original_allocation);
        }
    }
}
