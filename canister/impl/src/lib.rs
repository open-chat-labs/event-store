mod env;
mod guards;
mod lifecycle;
mod memory;
mod model;
mod queries;
mod state;
mod updates;

#[cfg(test)]
mod generate_candid_file {
    use event_sink_canister::*;
    use ic_cdk::export_candid;
    use std::env;
    use std::fs::write;
    use std::path::PathBuf;

    #[test]
    fn save_candid() {
        let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let dir = dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("canister")
            .join("api");

        export_candid!();
        write(dir.join("can.did"), __export_service()).unwrap()
    }
}
