use std::path::Path;

use anyhow::{anyhow, bail, Result};
use tokenizers::{Decoder, EncodeInput, Tokenizer};

use crate::config::{Config, Device};

const TOKENIZER_FILENAME: &str = "tokenizer.json";

mod translator;

/// A text translator with a tokenizer.
pub struct Translator {
    translator: self::translator::Translator,
    tokenizer: Tokenizer,
}

impl Translator {
    /// Initializes the translator and tokenizer.
    pub fn new<T: AsRef<Path>>(path: T, device: Device, config: Config) -> Result<Translator> {
        Translator::with_tokenizer(
            &path,
            device,
            config,
            Tokenizer::from_file(path.as_ref().join(TOKENIZER_FILENAME))
                .map_err(|err| anyhow!("failed to load a tokenizer: {err}"))?,
        )
    }

    /// Initializes the translator and tokenizer.
    pub fn with_tokenizer<T: AsRef<Path>>(
        path: T,
        device: Device,
        config: Config,
        tokenizer: Tokenizer,
    ) -> Result<Translator> {
        Ok(Translator {
            translator: self::translator::Translator::new(
                path.as_ref().to_str().unwrap(),
                device,
                config,
            )?,
            tokenizer,
        })
    }

    /// Translates a batch of strings.
    pub fn translate_batch<'a, T, U, V>(
        &self,
        sources: Vec<T>,
        target_prefixes: Vec<Vec<U>>,
        options: &TranslationOptions<V>,
    ) -> Result<Vec<(String, Option<f32>)>>
    where
        T: Into<EncodeInput<'a>>,
        U: AsRef<str>,
        V: AsRef<str>,
    {
        let tokens = sources
            .into_iter()
            .map(|s| {
                self.tokenizer
                    .encode(s, true)
                    .map(|r| r.get_tokens().to_vec())
                    .map_err(|err| anyhow!("failed to encode the given input: {err}"))
            })
            .collect::<Result<Vec<Vec<String>>>>()?;

        let output = self
            .translator
            .translate_batch(&tokens, &target_prefixes, options)?;

        let decoder = self.tokenizer.get_decoder().unwrap();
        let mut res = Vec::new();
        for (r, prefix) in output.into_iter().zip(target_prefixes) {
            let score = r.score();
            match r.hypotheses.into_iter().next() {
                None => bail!("no results are returned"),
                Some(h) => {
                    res.push((
                        decoder
                            .decode(h.into_iter().skip(prefix.len()).collect())
                            .map_err(|err| anyhow!("failed to decode: {err}"))?,
                        score,
                    ));
                }
            }
        }
        Ok(res)
    }
}

/// A text translator options.
pub struct TranslationOptions<'a> {
    /// Suppress sequences.
    pub suppress_sequences: Vec<Vec<&'a str>>,
}