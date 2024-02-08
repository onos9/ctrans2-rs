// generator.rs
//
// Copyright (c) 2023 Junpei Kawamoto
//
// This software is released under the MIT License.
//
// http://opensource.org/licenses/mit-license.php

//! Bindings for ctranslate2::Generator.

use cxx::UniquePtr;

use crate::config::{BatchType, ComputeType, Config, Device};

#[cxx::bridge]
mod ffi {
    struct GenVecStr<'a> {
        v: Vec<&'a str>,
    }

    struct GenVecString {
        v: Vec<String>,
    }

    struct GenVecUSize {
        v: Vec<usize>,
    }

    enum GenComputeType {
        Default,
        Auto,
        Float32,
        Int8,
        Int8Float16,
        Int16,
        Float16,
    }

    struct GeneratorConfig {
        compute_type: GenComputeType,
        device_indices: Vec<i32>,
        num_threads_per_replica: usize,
        max_queued_batches: i64,
        cpu_core_offset: i32,
    }

    enum GenerationBatchType {
        Examples,
        Tokens,
    }

    struct GenerationOptions<'a> {
        beam_size: usize,
        patience: f32,
        length_penalty: f32,
        repetition_penalty: f32,
        no_repeat_ngram_size: usize,
        disable_unk: bool,
        suppress_sequences: Vec<GenVecStr<'a>>,
        return_end_token: bool,
        max_length: usize,
        min_length: usize,
        sampling_topk: usize,
        sampling_topp: f32,
        sampling_temperature: f32,
        num_hypotheses: usize,
        return_scores: bool,
        return_alternatives: bool,
        min_alternative_expansion_prob: f32,
        static_prompt: Vec<&'a str>,
        cache_static_prompt: bool,
        include_prompt_in_result: bool,
        max_batch_size: usize,
        batch_type: GenerationBatchType,
    }

    struct GenerationResult {
        sequences: Vec<GenVecString>,
        sequences_ids: Vec<GenVecUSize>,
        scores: Vec<f32>,
    }

    unsafe extern "C++" {
        include!("ctranslate2/include/generator.h");

        type Generator;

        fn new_generator(
            model_path: &str,
            cuda: bool,
            config: GeneratorConfig,
        ) -> Result<UniquePtr<Generator>>;

        fn generate_batch(
            &self,
            start_tokens: Vec<GenVecStr>,
            options: GenerationOptions,
        ) -> Result<Vec<GenerationResult>>;
    }
}

/// A text translator.
pub struct Generator {
    ptr: UniquePtr<ffi::Generator>,
}

impl Generator {
    pub fn new<T: AsRef<str>>(
        model_path: T,
        device: Device,
        config: Config,
    ) -> anyhow::Result<Generator> {
        Ok(Generator {
            ptr: ffi::new_generator(
                model_path.as_ref(),
                match device {
                    Device::CPU => false,
                    Device::CUDA => true,
                },
                ffi::GeneratorConfig {
                    compute_type: match config.compute_type {
                        ComputeType::Default => ffi::GenComputeType::Default,
                        ComputeType::Auto => ffi::GenComputeType::Auto,
                        ComputeType::Float32 => ffi::GenComputeType::Float32,
                        ComputeType::Int8 => ffi::GenComputeType::Int8,
                        ComputeType::Int8Float16 => ffi::GenComputeType::Int8Float16,
                        ComputeType::Int16 => ffi::GenComputeType::Int16,
                        ComputeType::Float16 => ffi::GenComputeType::Float16,
                    },
                    device_indices: config.device_indices,
                    num_threads_per_replica: config.num_threads_per_replica,
                    max_queued_batches: config.max_queued_batches,
                    cpu_core_offset: config.cpu_core_offset,
                },
            )?,
        })
    }

    /// Generates from a batch of start tokens.
    ///
    /// `start_tokens` are Batch of start tokens. If the decoder starts from a special start token
    /// like `<s>`, this token should be added to this input.
    pub fn generate_batch<T: AsRef<str>, U: AsRef<str>, V: AsRef<str>>(
        &self,
        start_tokens: &[Vec<T>],
        options: &GenerationOptions<U, V>,
    ) -> anyhow::Result<Vec<GenerationResult>> {
        Ok(self
            .ptr
            .generate_batch(vec_ffi_vecstr(start_tokens), options.to_ffi())?
            .into_iter()
            .map(GenerationResult::from)
            .collect())
    }
}

/// The set of generation options.
#[derive(Debug)]
pub struct GenerationOptions<T: AsRef<str>, U: AsRef<str>> {
    /// Beam size to use for beam search (set 1 to run greedy search).
    pub beam_size: usize,
    /// Beam search patience factor, as described in <https://arxiv.org/abs/2204.05424>.
    /// The decoding will continue until beam_size*patience hypotheses are finished.
    pub patience: f32,
    /// Exponential penalty applied to the length during beam search.
    /// The scores are normalized with:
    /// ```math
    ///   hypothesis_score /= (hypothesis_length ** length_penalty)
    /// ```
    pub length_penalty: f32,
    /// Penalty applied to the score of previously generated tokens, as described in
    /// <https://arxiv.org/abs/1909.05858> (set > 1 to penalize).
    pub repetition_penalty: f32,
    /// Prevent repetitions of ngrams with this size (set 0 to disable).
    pub no_repeat_ngram_size: usize,
    /// Disable the generation of the unknown token.
    pub disable_unk: bool,
    /// Disable the generation of some sequences of tokens.
    pub suppress_sequences: Vec<Vec<T>>,
    // Stop the decoding on one of these tokens (defaults to the model EOS token).
    //std::variant<std::string, std::vector<std::string>, std::vector<size_t>> end_token;
    /// Include the end token in the result.
    pub return_end_token: bool,
    /// Length constraints.
    pub max_length: usize,
    /// Length constraints.
    pub min_length: usize,
    /// Randomly sample from the top K candidates (set 0 to sample from the full output distribution).
    pub sampling_topk: usize,
    /// Keep the most probable tokens whose cumulative probability exceeds this value.
    pub sampling_topp: f32,
    /// High temperature increase randomness.
    pub sampling_temperature: f32,
    /// Number of hypotheses to include in the result.
    pub num_hypotheses: usize,
    /// Include scores in the result.
    pub return_scores: bool,
    /// Return alternatives at the first unconstrained decoding position. This is typically
    /// used with a prefix to provide alternatives at a specifc location.
    pub return_alternatives: bool,
    /// Minimum probability to expand an alternative.
    pub min_alternative_expansion_prob: f32,
    /// The static prompt will prefix all inputs for this model.
    pub static_prompt: Vec<U>,
    /// Cache the model state after the static prompt and reuse it for future runs using
    /// the same static prompt.
    pub cache_static_prompt: bool,
    /// Include the input tokens in the generation result.
    pub include_prompt_in_result: bool,
    // Function to call for each generated token in greedy search.
    // Returns true indicate the current generation is considered finished thus can be stopped early.
    //std::function<bool(GenerationStepResult)> callback = nullptr;
    /// The maximum batch size. If the number of inputs is greater than `max_batch_size`,
    /// the inputs are sorted by length and split by chunks of `max_batch_size` examples
    /// so that the number of padding positions is minimized.
    pub max_batch_size: usize,
    /// Whether `max_batch_size` is the number of `examples` or `tokens`.
    pub batch_type: BatchType,
}

impl Default for GenerationOptions<String, String> {
    fn default() -> Self {
        Self {
            beam_size: 1,
            patience: 1.,
            length_penalty: 1.,
            repetition_penalty: 1.,
            no_repeat_ngram_size: 0,
            disable_unk: false,
            suppress_sequences: vec![],
            return_end_token: false,
            max_length: 512,
            min_length: 0,
            sampling_topk: 1,
            sampling_topp: 1.,
            sampling_temperature: 1.,
            num_hypotheses: 1,
            return_scores: false,
            return_alternatives: false,
            min_alternative_expansion_prob: 0.,
            static_prompt: vec![],
            cache_static_prompt: true,
            include_prompt_in_result: true,
            max_batch_size: 0,
            batch_type: Default::default(),
        }
    }
}

impl<T: AsRef<str>, U: AsRef<str>> GenerationOptions<T, U> {
    #[inline]
    fn to_ffi(&self) -> ffi::GenerationOptions {
        ffi::GenerationOptions {
            beam_size: self.beam_size,
            patience: self.patience,
            length_penalty: self.length_penalty,
            repetition_penalty: self.repetition_penalty,
            no_repeat_ngram_size: self.no_repeat_ngram_size,
            disable_unk: self.disable_unk,
            suppress_sequences: vec_ffi_vecstr(self.suppress_sequences.as_ref()),
            return_end_token: self.return_end_token,
            max_length: self.max_length,
            min_length: self.min_length,
            sampling_topk: self.sampling_topk,
            sampling_topp: self.sampling_topp,
            sampling_temperature: self.sampling_temperature,
            num_hypotheses: self.num_hypotheses,
            return_scores: self.return_scores,
            return_alternatives: self.return_alternatives,
            min_alternative_expansion_prob: self.min_alternative_expansion_prob,
            static_prompt: self.static_prompt.iter().map(|v| v.as_ref()).collect(),
            cache_static_prompt: self.cache_static_prompt,
            include_prompt_in_result: self.include_prompt_in_result,
            max_batch_size: self.max_batch_size,
            batch_type: match self.batch_type {
                BatchType::Examples => ffi::GenerationBatchType::Examples,
                BatchType::Tokens => ffi::GenerationBatchType::Tokens,
            },
        }
    }
}

/// A generation result.
#[derive(Debug)]
pub struct GenerationResult {
    /// Generated sequences of tokens.
    pub sequences: Vec<Vec<String>>,
    /// Generated sequences of token IDs.
    pub sequences_ids: Vec<Vec<usize>>,
    /// Score of each sequence (empty if `return_scores` was disabled).
    pub scores: Vec<f32>,
}

impl From<ffi::GenerationResult> for GenerationResult {
    fn from(res: ffi::GenerationResult) -> Self {
        Self {
            sequences: res.sequences.into_iter().map(|c| c.v).collect(),
            sequences_ids: res.sequences_ids.into_iter().map(|c| c.v).collect(),
            scores: res.scores,
        }
    }
}

impl GenerationResult {
    /// Returns the number of sequences.
    pub fn num_sequences(&self) -> usize {
        self.sequences.len()
    }

    /// Returns true if this result has scores.
    pub fn has_scores(&self) -> bool {
        !self.scores.is_empty()
    }
}

#[inline]
fn vec_ffi_vecstr<T: AsRef<str>>(src: &[Vec<T>]) -> Vec<ffi::GenVecStr> {
    src.iter()
        .map(|v| ffi::GenVecStr {
            v: v.iter().map(|s| s.as_ref()).collect(),
        })
        .collect()
}
