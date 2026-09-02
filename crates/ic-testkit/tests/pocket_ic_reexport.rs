use ic_testkit::{
    pic,
    pocket_ic::{
        CanisterSettings, CreateCanisterParams, PocketIc,
        common::rest::{BlobCompression, IcpFeatures, IcpFeaturesConfig},
    },
};

#[test]
fn complete_upstream_crate_is_available_through_ic_testkit() {
    fn exported<T>() {}

    exported::<CanisterSettings>();
    exported::<CreateCanisterParams>();
    exported::<BlobCompression>();
    exported::<IcpFeatures>();
    exported::<IcpFeaturesConfig>();

    let upstream: Option<PocketIc> = None;
    let convenience: Option<pic::PocketIc> = upstream;
    let _: Option<PocketIc> = convenience;
}
