use frame_support::weights::Weight;

pub trait WeightInfo {
    fn sample_final_call() -> Weight;
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn sample_final_call() -> Weight {
        Weight::from_ref_time(17_443_346 as u64)
    }
}
