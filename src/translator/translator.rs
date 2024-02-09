use cxx::UniquePtr;

/// Model computation type or a dictionary mapping a device name to the computation type.
#[cxx::bridge]
mod ffi {

    struct TransVecStr<'a> {
        v: Vec<&'a str>,
    }

    struct TransVecString {
        v: Vec<String>,
    }

    struct TransVecUSize {
        v: Vec<usize>,
    }

    pub enum ComputeType {
        Default,
        Auto,
        Float32,
        Int8,
        Int8Float16,
        Int16,
        Float16,
    }

    pub struct Config {
        /// Model computation type or a dictionary mapping a device name to the computation type.
        pub compute_type: ComputeType,
        /// Device IDs where to place this generator on.
        pub device_indices: Vec<i32>,
        pub num_threads_per_replica: usize,
        pub max_queued_batches: i64,
        pub cpu_core_offset: i32,
    }

    pub struct TranslationOptions<'a> {
        suppress_sequences: Vec<TransVecStr<'a>>,
    }

    struct TraslationResult {
        sequences: Vec<TransVecString>,
        sequences_ids: Vec<TransVecUSize>,
        scores: Vec<f32>,
    }

    /// Device to use.
    #[derive(Debug)]
    pub enum Device {
        CPU,
        CUDA,
    }

    unsafe extern "C++" {
        include!("../../include/translator.h");

        type Translator;

        fn new_translator(
            model_path: &str,
            cuda: bool,
            config: Config,
        ) -> Result<UniquePtr<Translator>>;

        fn translate_batch(
            &self,
            source: Vec<TransVecStr>,
            target_prefix: Vec<TransVecStr>,
            options: TranslationOptions,
        ) -> Result<Vec<TraslationResult>>;
    }
}

pub struct Translator {
    ptr: UniquePtr<ffi::Translator>,
}

// impl Default for Config {
//     fn default() -> Self {
//         Self {
//             compute_type: Default::default(),
//             device_indices: vec![0],
//             num_threads_per_replica: 0,
//             max_queued_batches: 0,
//             cpu_core_offset: -1,
//         }
//     }
// }

/// Whether max_batch_size is the number of “examples” or “tokens”.
#[derive(Debug, Default)]
pub enum BatchType {
    #[default]
    Examples,
    Tokens,
}
