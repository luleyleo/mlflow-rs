mod client;
mod experiment;
mod run;
mod storage;

pub use client::Client;
pub use experiment::Experiment;
pub use run::Run;

type Id = String;

pub fn time_stamp() -> u64 {
    use std::convert::TryInto;
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap()
}
