use std::path::Path;

use anyhow::{anyhow, Result};
use tokenizers::{Decoder, EncodeInput, Tokenizer};
use crate::config::{Config, Device};
use self::generator::GenerationOptions;

mod generator;

const TOKENIZER_FILENAME: &str = "tokenizer.json";

pub struct Generator {
    generator: self::generator::Generator,
    tokenizer: Tokenizer,
}

impl Generator {
    /// Initializes the generator and tokenizer.
    pub fn new<T: AsRef<Path>>(path: T, device: Device, config: Config) -> Result<Generator> {
        Generator::with_tokenizer(
            &path,
            device,
            config,
            Tokenizer::from_file(path.as_ref().join(TOKENIZER_FILENAME))
                .map_err(|err| anyhow!("failed to load a tokenizer: {err}"))?,
        )
    }

    /// Initializes the generator with the given tokenizer.
    pub fn with_tokenizer<T: AsRef<Path>>(
        path: T,
        device: Device,
        config: Config,
        tokenizer: Tokenizer,
    ) -> Result<Generator> {
        Ok(Generator {
            generator: self::generator::Generator::new(path.as_ref().to_str().unwrap(), device, config)?,
            tokenizer,
        })
    }

    /// Generate texts with the given prompts.
    pub fn generate_batch<'a, T, U, V>(
        &self,
        prompts: Vec<T>,
        options: &GenerationOptions<U, V>,
    ) -> Result<Vec<(Vec<String>, Vec<f32>)>>
    where
        T: Into<EncodeInput<'a>>,
        U: AsRef<str>,
        V: AsRef<str>,
    {
        let tokens = prompts
            .into_iter()
            .map(|s| {
                self.tokenizer
                    .encode(s, false)
                    .map(|r| r.get_tokens().to_vec())
                    .map_err(|err| anyhow!("failed to encode the given input: {err}"))
            })
            .collect::<Result<Vec<Vec<String>>>>()?;

        let output = self.generator.generate_batch(&tokens, options)?;

        let decoder = self.tokenizer.get_decoder().unwrap();
        let mut res = Vec::new();
        for r in output.into_iter() {
            let sequence = r
                .sequences
                .into_iter()
                .map(|seq| decoder.decode(seq))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| anyhow!("failed to decode: {err}"))?;
            let scores = r.scores;
            res.push((sequence, scores))
        }
        Ok(res)
    }
}
