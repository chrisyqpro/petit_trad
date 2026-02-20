// SPDX-License-Identifier: GPL-3.0-or-later

//! Model management for llama.cpp inference

use crate::{Config, Error, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use std::ffi::CStr;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::num::NonZeroU32;
use std::os::raw::c_char;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// Manages model loading and inference via llama.cpp
pub struct ModelManager {
    model: LlamaModel,
    backend: LlamaBackend,
    config: Config,
}

static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

impl ModelManager {
    /// Create a new ModelManager by loading a GGUF model
    pub fn new(config: Config) -> Result<Self> {
        // Validate model file exists
        if !config.model_path.exists() {
            return Err(Error::ModelLoad(format!(
                "Model file not found: {}",
                config.model_path.display()
            )));
        }

        // Initialize backend
        let mut backend =
            LlamaBackend::init().map_err(|e| Error::ModelLoad(format!("Backend init: {e}")))?;
        configure_logging(&config, &mut backend)?;

        // Set up model parameters
        let model_params = LlamaModelParams::default().with_n_gpu_layers(config.gpu_layers);

        // Load the model
        let model = LlamaModel::load_from_file(&backend, &config.model_path, &model_params)
            .map_err(|e| Error::ModelLoad(format!("Model load: {e}")))?;

        Ok(Self {
            model,
            backend,
            config,
        })
    }

    /// Run inference on a prompt and return the generated text
    pub fn infer(&self, prompt: &str, max_new_tokens: u32) -> Result<String> {
        // Create context for this inference
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(self.config.context_size))
            .with_n_threads(self.config.threads as i32)
            .with_n_threads_batch(self.config.threads as i32);

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| Error::Inference(format!("Context creation: {e}")))?;

        // Tokenize prompt
        let tokens = self
            .model
            .str_to_token(prompt, llama_cpp_2::model::AddBos::Always)
            .map_err(|e| Error::Inference(format!("Tokenization: {e}")))?;

        // Create batch and add tokens
        let mut batch = LlamaBatch::new(self.config.context_size as usize, 1);
        let last_idx = tokens.len() - 1;
        for (i, token) in tokens.iter().enumerate() {
            batch
                .add(*token, i as i32, &[0], i == last_idx)
                .map_err(|e| Error::Inference(format!("Batch add: {e}")))?;
        }

        // Process prompt
        ctx.decode(&mut batch)
            .map_err(|e| Error::Inference(format!("Decode prompt: {e}")))?;

        // Generate tokens
        let mut output_tokens = Vec::new();
        let mut n_cur = tokens.len();

        for _ in 0..max_new_tokens {
            // Sample next token (greedy: temperature 0)
            let candidates = ctx.candidates_ith(batch.n_tokens() - 1);
            let mut candidates_array = LlamaTokenDataArray::from_iter(candidates, false);
            let next_token = candidates_array.sample_token_greedy();

            // Check for end of generation
            if self.model.is_eog_token(next_token) {
                break;
            }

            output_tokens.push(next_token);

            // Prepare next batch
            batch.clear();
            batch
                .add(next_token, n_cur as i32, &[0], true)
                .map_err(|e| Error::Inference(format!("Batch add: {e}")))?;

            n_cur += 1;

            // Decode
            ctx.decode(&mut batch)
                .map_err(|e| Error::Inference(format!("Decode: {e}")))?;
        }

        // Convert tokens to string
        let mut output = String::new();
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        for token in output_tokens {
            let piece = self
                .model
                .token_to_piece(token, &mut decoder, true, None)
                .map_err(|e| Error::Inference(format!("Detokenization: {e}")))?;
            output.push_str(&piece);
        }

        Ok(output.trim().to_string())
    }

    /// Get a reference to the underlying model
    pub fn model(&self) -> &LlamaModel {
        &self.model
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

fn configure_logging(config: &Config, backend: &mut LlamaBackend) -> Result<()> {
    if config.log_to_file {
        init_log_file(&config.log_path)?;
        unsafe {
            llama_cpp_sys_2::llama_log_set(Some(log_callback), std::ptr::null_mut());
            llama_cpp_sys_2::ggml_log_set(Some(log_callback), std::ptr::null_mut());
        }
    } else {
        backend.void_logs();
    }
    Ok(())
}

fn init_log_file(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::ModelLoad(format!("Log dir create: {e}")))?;
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| Error::ModelLoad(format!("Log file open: {e}")))?;

    let _ = LOG_FILE.set(Mutex::new(file));
    Ok(())
}

unsafe extern "C" fn log_callback(
    _level: llama_cpp_sys_2::ggml_log_level,
    text: *const c_char,
    _user_data: *mut ::std::os::raw::c_void,
) {
    if text.is_null() {
        return;
    }
    let file_lock = match LOG_FILE.get() {
        Some(lock) => lock,
        None => return,
    };
    let mut file = match file_lock.lock() {
        Ok(file) => file,
        Err(_) => return,
    };
    // SAFETY: `text` is provided by llama.cpp for the duration of this callback and
    // was checked for null above.
    let message = unsafe { CStr::from_ptr(text) }.to_bytes();
    let _ = file.write_all(message);
}
