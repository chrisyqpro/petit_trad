// SPDX-License-Identifier: GPL-3.0-or-later

//! Glossary constraint subsystem.

use crate::config::GlossaryConfig;
use crate::language::normalize_lang;
use crate::{Error, Result};
use csv::{ReaderBuilder, StringRecord};
use fastembed::{
    EmbeddingModel, InitOptionsUserDefined, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use hnsw_rs::anndists::dist::distances::DistCosine;
use hnsw_rs::hnsw::Hnsw;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

const ANN_SEARCH_K: usize = 8;
const ANN_SEARCH_EF: usize = 32;
const ANN_SIMILARITY_THRESHOLD: f32 = 0.35;
const HNSW_MAX_CONNECTIONS: usize = 16;
const HNSW_MAX_LAYER: usize = 12;
const HNSW_EF_CONSTRUCTION: usize = 64;
const EMBEDDING_MODEL_CODE: &str = "onnx-community/embeddinggemma-300m-ONNX";
const EMBEDDING_MODEL_FILE: &str = "onnx/model.onnx";
const EMBEDDING_MODEL_DATA_FILE: &str = "onnx/model.onnx_data";
const EMBEDDING_TOKENIZER_FILE: &str = "tokenizer.json";
const EMBEDDING_CONFIG_FILE: &str = "config.json";
const EMBEDDING_SPECIAL_TOKENS_FILE: &str = "special_tokens_map.json";
const EMBEDDING_TOKENIZER_CONFIG_FILE: &str = "tokenizer_config.json";

type LangPairKey = (String, String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryCandidate {
    pub source_term: String,
    pub target_term: String,
}

#[derive(Default)]
pub struct GlossaryStore {
    pair_indices: HashMap<LangPairKey, PairGlossaryIndex>,
    max_matches: usize,
    provider: Option<Arc<dyn EmbeddingProvider>>,
}

impl std::fmt::Debug for GlossaryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlossaryStore")
            .field("pair_indices", &self.pair_indices.len())
            .field("max_matches", &self.max_matches)
            .field("has_provider", &self.provider.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GlossaryRow {
    source_lang: String,
    target_lang: String,
    source_term: String,
    target_term: String,
    note: Option<String>,
    source_term_norm: String,
}

struct PairGlossaryIndex {
    entries: Vec<GlossaryEntry>,
    hnsw: Hnsw<'static, f32, DistCosine>,
}

struct GlossaryEntry {
    source_term: String,
    target_term: String,
    source_term_norm: String,
}

trait EmbeddingProvider: Send + Sync {
    fn embed_passages(&self, passages: &[String]) -> Result<Vec<Vec<f32>>>;
    fn embed_query(&self, query: &str) -> Result<Vec<f32>>;
}

struct FastEmbedProvider {
    model: Mutex<TextEmbedding>,
}

impl FastEmbedProvider {
    fn new(model_dir: &Path) -> Result<Self> {
        let model = load_embedding_model(model_dir)?;
        let options = InitOptionsUserDefined::new();
        let model = TextEmbedding::try_new_from_user_defined(model, options)
            .map_err(|err| Error::GlossaryEmbeddingInit(err.to_string()))?;
        Ok(Self {
            model: Mutex::new(model),
        })
    }

    fn lock_model(&self) -> Result<std::sync::MutexGuard<'_, TextEmbedding>> {
        self.model.lock().map_err(|_| {
            Error::GlossaryEmbeddingGenerate("fastembed model lock poisoned".to_string())
        })
    }
}

impl EmbeddingProvider for FastEmbedProvider {
    fn embed_passages(&self, passages: &[String]) -> Result<Vec<Vec<f32>>> {
        let encoded = passages
            .iter()
            .map(|passage| format!("passage: {passage}"))
            .collect::<Vec<_>>();
        let mut model = self.lock_model()?;
        model
            .embed(encoded, None)
            .map_err(|err| Error::GlossaryEmbeddingGenerate(err.to_string()))
    }

    fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        let mut model = self.lock_model()?;
        let embeddings = model
            .embed([format!("query: {query}")], None)
            .map_err(|err| Error::GlossaryEmbeddingGenerate(err.to_string()))?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| Error::GlossaryEmbeddingGenerate("empty query embedding".to_string()))
    }
}

impl GlossaryStore {
    pub fn from_config(config: &GlossaryConfig) -> Result<Self> {
        if !config.enabled {
            return Ok(Self {
                pair_indices: HashMap::new(),
                max_matches: config.max_matches,
                provider: None,
            });
        }

        if !config.path.exists() {
            return Err(Error::GlossaryRead(format!(
                "missing glossary file: {}",
                config.path.display()
            )));
        }

        let rows = load_rows_from_path(&config.path)?;
        let provider = Arc::new(FastEmbedProvider::new(&config.embedding_model_dir)?);
        build_store_from_rows(rows, provider, config.max_matches)
    }

    pub fn select_candidates(
        &self,
        source_lang: &str,
        target_lang: &str,
        source_text: &str,
    ) -> Result<Vec<GlossaryCandidate>> {
        if self.max_matches == 0 {
            return Ok(Vec::new());
        }

        let provider = match &self.provider {
            Some(provider) => provider,
            None => return Ok(Vec::new()),
        };

        let key = lang_pair_key(source_lang, target_lang);
        let index = match self.pair_indices.get(&key) {
            Some(index) => index,
            None => return Ok(Vec::new()),
        };

        let normalized_source_text = normalize_source_text(source_text);
        if normalized_source_text.is_empty() {
            return Ok(Vec::new());
        }

        let query_embedding = provider.embed_query(source_text)?;

        let mut exact = Vec::new();
        for entry in &index.entries {
            if normalized_source_text.contains(&entry.source_term_norm) {
                exact.push(RankedCandidate::from_exact(entry));
            }
        }
        exact.sort_by(compare_exact_candidates);

        let mut ann = Vec::new();
        if !index.entries.is_empty() {
            let knbn = ANN_SEARCH_K.min(index.entries.len()).max(1);
            let ef = ANN_SEARCH_EF.max(knbn + 1);
            let neighbours = index.hnsw.search(query_embedding.as_slice(), knbn, ef);

            for neighbour in neighbours {
                let origin_id = neighbour.get_origin_id();
                let Some(entry) = index.entries.get(origin_id) else {
                    return Err(Error::GlossaryIndexBuild(format!(
                        "search returned invalid origin id: {origin_id}"
                    )));
                };

                let similarity = 1.0 - neighbour.get_distance();
                if similarity >= ANN_SIMILARITY_THRESHOLD {
                    ann.push(RankedCandidate::from_ann(entry, similarity));
                }
            }
        }

        ann.sort_by(compare_ann_candidates);

        let mut shortlist = Vec::new();
        let mut seen_terms = HashSet::new();
        for candidate in exact.into_iter().chain(ann.into_iter()) {
            if seen_terms.insert(candidate.source_term_norm.clone()) {
                shortlist.push(candidate.into_public());
                if shortlist.len() == self.max_matches {
                    break;
                }
            }
        }

        Ok(shortlist)
    }
}

fn load_embedding_model(model_dir: &Path) -> Result<UserDefinedEmbeddingModel> {
    ensure_embedding_model_dir(model_dir)?;

    let onnx_file = read_embedding_model_file(model_dir, EMBEDDING_MODEL_FILE)?;
    let onnx_data_file = read_embedding_model_file(model_dir, EMBEDDING_MODEL_DATA_FILE)?;
    let tokenizer_files = TokenizerFiles {
        tokenizer_file: read_embedding_model_file(model_dir, EMBEDDING_TOKENIZER_FILE)?,
        config_file: read_embedding_model_file(model_dir, EMBEDDING_CONFIG_FILE)?,
        special_tokens_map_file: read_embedding_model_file(
            model_dir,
            EMBEDDING_SPECIAL_TOKENS_FILE,
        )?,
        tokenizer_config_file: read_embedding_model_file(
            model_dir,
            EMBEDDING_TOKENIZER_CONFIG_FILE,
        )?,
    };

    let model_info = TextEmbedding::get_model_info(&EmbeddingModel::EmbeddingGemma300M)
        .map_err(|err| Error::GlossaryEmbeddingInit(err.to_string()))?;
    let mut model = UserDefinedEmbeddingModel::new(onnx_file, tokenizer_files)
        .with_quantization(TextEmbedding::get_quantization_mode(
            &EmbeddingModel::EmbeddingGemma300M,
        ))
        .with_external_initializer(
            Path::new(EMBEDDING_MODEL_DATA_FILE)
                .file_name()
                .expect("static file name should have a basename")
                .to_string_lossy()
                .into_owned(),
            onnx_data_file,
        );
    if let Some(pooling) =
        TextEmbedding::get_default_pooling_method(&EmbeddingModel::EmbeddingGemma300M)
    {
        model = model.with_pooling(pooling);
    }
    model.output_key = model_info.output_key.clone();

    Ok(model)
}

fn ensure_embedding_model_dir(model_dir: &Path) -> Result<()> {
    if model_dir.is_dir() {
        return Ok(());
    }

    if model_dir.exists() {
        return Err(Error::GlossaryEmbeddingInit(format!(
            "embedding model directory is not a directory: {}",
            model_dir.display()
        )));
    }

    Err(Error::GlossaryEmbeddingInit(format!(
        "missing embedding model directory for {EMBEDDING_MODEL_CODE}: {}",
        model_dir.display()
    )))
}

fn read_embedding_model_file(model_dir: &Path, relative_path: &str) -> Result<Vec<u8>> {
    let path = model_dir.join(relative_path);
    fs::read(&path).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => Error::GlossaryEmbeddingInit(format!(
            "missing embedding model file for {EMBEDDING_MODEL_CODE}: {}",
            path.display()
        )),
        _ => Error::GlossaryEmbeddingInit(format!(
            "failed to read embedding model file {}: {err}",
            path.display()
        )),
    })
}

fn build_store_from_rows<P>(
    rows: Vec<GlossaryRow>,
    provider: Arc<P>,
    max_matches: usize,
) -> Result<GlossaryStore>
where
    P: EmbeddingProvider + 'static,
{
    if rows.is_empty() {
        let provider: Arc<dyn EmbeddingProvider> = provider;
        return Ok(GlossaryStore {
            pair_indices: HashMap::new(),
            max_matches,
            provider: Some(provider),
        });
    }

    let first_source_term = rows[0].source_term.clone();
    provider.embed_query(&first_source_term)?;

    let mut grouped: BTreeMap<LangPairKey, Vec<GlossaryRow>> = BTreeMap::new();
    for row in rows {
        grouped
            .entry((row.source_lang.clone(), row.target_lang.clone()))
            .or_default()
            .push(row);
    }

    let mut pair_indices = HashMap::new();
    for (pair, grouped_rows) in grouped {
        let source_terms = grouped_rows
            .iter()
            .map(|row| row.source_term.clone())
            .collect::<Vec<_>>();
        let passage_embeddings = provider.embed_passages(&source_terms)?;
        if passage_embeddings.len() != grouped_rows.len() {
            return Err(Error::GlossaryIndexBuild(format!(
                "embedding count mismatch for {}->{}",
                pair.0, pair.1
            )));
        }

        let expected_dims = passage_embeddings
            .first()
            .map(|vector| vector.len())
            .unwrap_or(0);
        if expected_dims == 0
            || passage_embeddings
                .iter()
                .any(|vector| vector.len() != expected_dims)
        {
            return Err(Error::GlossaryIndexBuild(
                "embedding dimensions mismatch".to_string(),
            ));
        }

        let mut hnsw = Hnsw::new(
            HNSW_MAX_CONNECTIONS,
            grouped_rows.len().max(1),
            HNSW_MAX_LAYER,
            HNSW_EF_CONSTRUCTION,
            DistCosine,
        );

        let mut entries = Vec::with_capacity(grouped_rows.len());
        for (idx, (row, embedding)) in grouped_rows.into_iter().zip(passage_embeddings).enumerate()
        {
            hnsw.insert_slice((embedding.as_slice(), idx));
            entries.push(GlossaryEntry {
                source_term: row.source_term,
                target_term: row.target_term,
                source_term_norm: row.source_term_norm,
            });
        }
        hnsw.set_searching_mode(true);

        pair_indices.insert(pair, PairGlossaryIndex { entries, hnsw });
    }

    let provider: Arc<dyn EmbeddingProvider> = provider;
    Ok(GlossaryStore {
        pair_indices,
        max_matches,
        provider: Some(provider),
    })
}

fn load_rows_from_path(path: &Path) -> Result<Vec<GlossaryRow>> {
    let content = fs::read_to_string(path)
        .map_err(|_| Error::GlossaryRead(format!("missing glossary file: {}", path.display())))?;
    parse_tsv_rows(&content)
}

fn parse_tsv_rows(tsv: &str) -> Result<Vec<GlossaryRow>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_reader(tsv.as_bytes());

    let headers = reader
        .headers()
        .map_err(|err| Error::GlossaryParse(format!("TSV parse error: {err}")))?
        .clone();

    let source_lang_idx = header_index(&headers, "source_lang")?;
    let target_lang_idx = header_index(&headers, "target_lang")?;
    let source_term_idx = header_index(&headers, "source_term")?;
    let target_term_idx = header_index(&headers, "target_term")?;
    let note_idx = headers.iter().position(|header| header == "note");

    let mut rows = Vec::new();
    let mut seen = HashSet::new();

    for record in reader.records() {
        let record =
            record.map_err(|err| Error::GlossaryParse(format!("TSV parse error: {err}")))?;
        let source_lang = required_record_field(&record, source_lang_idx, "source_lang")?;
        let target_lang = required_record_field(&record, target_lang_idx, "target_lang")?;
        let source_term = required_record_field(&record, source_term_idx, "source_term")?;
        let target_term = required_record_field(&record, target_term_idx, "target_term")?;
        let note = note_idx
            .and_then(|idx| record.get(idx))
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string());

        let source_lang = normalize_lang(&source_lang);
        let target_lang = normalize_lang(&target_lang);
        let source_term_norm = normalize_source_text(&source_term);
        let target_term = target_term.trim().to_string();
        let key = (
            source_lang.clone(),
            target_lang.clone(),
            source_term_norm.clone(),
            target_term.clone(),
        );

        if seen.insert(key) {
            rows.push(GlossaryRow {
                source_lang,
                target_lang,
                source_term: source_term.trim().to_string(),
                target_term,
                note,
                source_term_norm,
            });
        }
    }

    Ok(rows)
}

fn header_index(headers: &StringRecord, header: &str) -> Result<usize> {
    headers
        .iter()
        .position(|candidate| candidate == header)
        .ok_or_else(|| Error::GlossaryParse(format!("missing required header: {header}")))
}

fn required_record_field(record: &StringRecord, idx: usize, field: &str) -> Result<String> {
    let value = record
        .get(idx)
        .ok_or_else(|| Error::GlossaryParse(format!("empty required field: {field}")))?;
    let value = value.trim();
    if value.is_empty() {
        return Err(Error::GlossaryParse(format!(
            "empty required field: {field}"
        )));
    }
    Ok(value.to_string())
}

fn lang_pair_key(source_lang: &str, target_lang: &str) -> LangPairKey {
    (normalize_lang(source_lang), normalize_lang(target_lang))
}

fn normalize_source_text(text: &str) -> String {
    text.split_whitespace()
        .map(|segment| segment.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Clone)]
struct RankedCandidate {
    source_term: String,
    target_term: String,
    source_term_norm: String,
    similarity: f32,
    exact_len: usize,
}

impl RankedCandidate {
    fn from_exact(entry: &GlossaryEntry) -> Self {
        Self {
            source_term: entry.source_term.clone(),
            target_term: entry.target_term.clone(),
            source_term_norm: entry.source_term_norm.clone(),
            similarity: 1.0,
            exact_len: entry.source_term_norm.len(),
        }
    }

    fn from_ann(entry: &GlossaryEntry, similarity: f32) -> Self {
        Self {
            source_term: entry.source_term.clone(),
            target_term: entry.target_term.clone(),
            source_term_norm: entry.source_term_norm.clone(),
            similarity,
            exact_len: entry.source_term_norm.len(),
        }
    }

    fn into_public(self) -> GlossaryCandidate {
        GlossaryCandidate {
            source_term: self.source_term,
            target_term: self.target_term,
        }
    }
}

fn compare_exact_candidates(left: &RankedCandidate, right: &RankedCandidate) -> Ordering {
    right
        .exact_len
        .cmp(&left.exact_len)
        .then_with(|| left.source_term.cmp(&right.source_term))
        .then_with(|| left.target_term.cmp(&right.target_term))
}

fn compare_ann_candidates(left: &RankedCandidate, right: &RankedCandidate) -> Ordering {
    right
        .similarity
        .partial_cmp(&left.similarity)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.source_term.cmp(&right.source_term))
        .then_with(|| left.target_term.cmp(&right.target_term))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct StubEmbeddingProvider {
        passage_vectors: HashMap<String, Vec<f32>>,
        query_vector: Vec<f32>,
        passage_error: Option<&'static str>,
        query_error: Option<&'static str>,
    }

    impl StubEmbeddingProvider {
        fn deterministic(vectors: &[(&str, &[f32])], query_vector: &[f32]) -> Self {
            let passage_vectors = vectors
                .iter()
                .map(|(term, vector)| ((*term).to_string(), vector.to_vec()))
                .collect();

            Self {
                passage_vectors,
                query_vector: query_vector.to_vec(),
                passage_error: None,
                query_error: None,
            }
        }

        fn failing_on_query(message: &'static str) -> Self {
            Self {
                passage_vectors: HashMap::new(),
                query_vector: vec![0.0],
                passage_error: None,
                query_error: Some(message),
            }
        }
    }

    impl EmbeddingProvider for StubEmbeddingProvider {
        fn embed_passages(&self, passages: &[String]) -> Result<Vec<Vec<f32>>> {
            if let Some(message) = self.passage_error {
                return Err(Error::GlossaryEmbeddingGenerate(message.to_string()));
            }

            Ok(passages
                .iter()
                .map(|passage| {
                    self.passage_vectors
                        .get(passage)
                        .cloned()
                        .unwrap_or_else(|| vec![0.0, 0.0])
                })
                .collect())
        }

        fn embed_query(&self, _query: &str) -> Result<Vec<f32>> {
            if let Some(message) = self.query_error {
                return Err(Error::GlossaryEmbeddingGenerate(message.to_string()));
            }

            Ok(self.query_vector.clone())
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("petit-trad-{name}-{stamp}.tsv"))
    }

    fn glossary_config(
        path: PathBuf,
        embedding_model_dir: PathBuf,
        max_matches: usize,
    ) -> GlossaryConfig {
        GlossaryConfig {
            enabled: true,
            path,
            embedding_model_dir,
            max_matches,
        }
    }

    #[test]
    fn glossary_parse_rejects_missing_headers() {
        let tsv = "\
source_lang\ttarget_lang\tsource_term\n\
en\tfr\taccount balance\n";

        let err = parse_tsv_rows(tsv).expect_err("missing TSV headers should fail");
        assert!(matches!(
            err,
            Error::GlossaryParse(message) if message == "missing required header: target_term"
        ));
    }

    #[test]
    fn glossary_parse_rejects_empty_required_fields() {
        let tsv = "\
source_lang\ttarget_lang\tsource_term\ttarget_term\n\
en\tfr\t\tsolde du compte\n";

        let err = parse_tsv_rows(tsv).expect_err("empty required fields should fail");
        assert!(matches!(
            err,
            Error::GlossaryParse(message) if message == "empty required field: source_term"
        ));
    }

    #[test]
    fn glossary_parse_keeps_note_and_dedupes_rows() {
        let tsv = "\
source_lang\ttarget_lang\tsource_term\ttarget_term\tnote\n\
en\tfr\taccount balance\tsolde du compte\tfinance\n\
EN\tFR\taccount  balance\tsolde du compte\trepeated\n\
en\tde\tbank account\tBankkonto\tbanking\n";

        let rows = parse_tsv_rows(tsv).expect("TSV should parse");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].note.as_deref(), Some("finance"));
        assert_eq!(rows[0].source_lang, "en");
        assert_eq!(rows[0].target_lang, "fr");
    }

    #[test]
    fn glossary_select_candidates_partitions_by_language_pair() {
        let rows = vec![
            GlossaryRow {
                source_lang: "en".into(),
                target_lang: "fr".into(),
                source_term: "account balance".into(),
                target_term: "solde du compte".into(),
                note: None,
                source_term_norm: normalize_source_text("account balance"),
            },
            GlossaryRow {
                source_lang: "en".into(),
                target_lang: "de".into(),
                source_term: "account balance".into(),
                target_term: "Kontostand".into(),
                note: None,
                source_term_norm: normalize_source_text("account balance"),
            },
        ];
        let provider = Arc::new(StubEmbeddingProvider::deterministic(
            &[
                ("account balance", &[1.0, 0.0]),
                ("Kontostand", &[0.0, 1.0]),
            ],
            &[1.0, 0.0],
        ));
        let store = build_store_from_rows(rows, provider, 4).expect("store should build");

        let french = store
            .select_candidates("en", "fr", "Your account balance is visible")
            .expect("pair should resolve");
        let german = store
            .select_candidates("en", "de", "Your account balance is visible")
            .expect("pair should resolve");

        assert_eq!(
            french,
            vec![GlossaryCandidate {
                source_term: "account balance".into(),
                target_term: "solde du compte".into(),
            }]
        );
        assert_eq!(
            german,
            vec![GlossaryCandidate {
                source_term: "account balance".into(),
                target_term: "Kontostand".into(),
            }]
        );
    }

    #[test]
    fn glossary_select_candidates_promotes_exact_matches_ahead_of_ann_only() {
        let rows = vec![
            GlossaryRow {
                source_lang: "en".into(),
                target_lang: "fr".into(),
                source_term: "account balance".into(),
                target_term: "solde du compte".into(),
                note: None,
                source_term_norm: normalize_source_text("account balance"),
            },
            GlossaryRow {
                source_lang: "en".into(),
                target_lang: "fr".into(),
                source_term: "balance sheet".into(),
                target_term: "bilan".into(),
                note: None,
                source_term_norm: normalize_source_text("balance sheet"),
            },
        ];
        let provider = Arc::new(StubEmbeddingProvider::deterministic(
            &[
                ("account balance", &[0.95, 0.05]),
                ("balance sheet", &[0.96, 0.04]),
            ],
            &[0.94, 0.06],
        ));
        let store = build_store_from_rows(rows, provider, 4).expect("store should build");

        let candidates = store
            .select_candidates("en", "fr", "The account balance is visible")
            .expect("selection should work");

        assert!(
            !candidates.is_empty(),
            "selection should produce at least one exact-match candidate"
        );
        assert_eq!(candidates[0].source_term, "account balance");
        assert_eq!(candidates[0].target_term, "solde du compte");
    }

    #[test]
    fn glossary_select_candidates_orders_ties_and_truncates_to_max_matches() {
        let rows = vec![
            GlossaryRow {
                source_lang: "en".into(),
                target_lang: "fr".into(),
                source_term: "alpha term".into(),
                target_term: "terme alpha".into(),
                note: None,
                source_term_norm: normalize_source_text("alpha term"),
            },
            GlossaryRow {
                source_lang: "en".into(),
                target_lang: "fr".into(),
                source_term: "beta term".into(),
                target_term: "terme beta".into(),
                note: None,
                source_term_norm: normalize_source_text("beta term"),
            },
            GlossaryRow {
                source_lang: "en".into(),
                target_lang: "fr".into(),
                source_term: "gamma term".into(),
                target_term: "terme gamma".into(),
                note: None,
                source_term_norm: normalize_source_text("gamma term"),
            },
        ];
        let provider = Arc::new(StubEmbeddingProvider::deterministic(
            &[
                ("alpha term", &[1.0, 0.0]),
                ("beta term", &[1.0, 0.0]),
                ("gamma term", &[1.0, 0.0]),
            ],
            &[1.0, 0.0],
        ));
        let store = build_store_from_rows(rows, provider, 2).expect("store should build");

        let candidates = store
            .select_candidates("en", "fr", "alpha beta gamma")
            .expect("selection should work");

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].source_term, "alpha term");
        assert_eq!(candidates[1].source_term, "beta term");
    }

    #[test]
    fn glossary_select_candidates_projects_only_source_and_target_terms() {
        let rows = vec![GlossaryRow {
            source_lang: "en".into(),
            target_lang: "fr".into(),
            source_term: "account balance".into(),
            target_term: "solde du compte".into(),
            note: Some("finance".into()),
            source_term_norm: normalize_source_text("account balance"),
        }];
        let provider = Arc::new(StubEmbeddingProvider::deterministic(
            &[("account balance", &[1.0, 0.0])],
            &[1.0, 0.0],
        ));
        let store = build_store_from_rows(rows, provider, 4).expect("store should build");

        let candidates = store
            .select_candidates("en", "fr", "account balance")
            .expect("selection should work");

        assert_eq!(
            candidates,
            vec![GlossaryCandidate {
                source_term: "account balance".into(),
                target_term: "solde du compte".into(),
            }]
        );
    }

    #[test]
    fn glossary_from_config_reports_missing_glossary_file() {
        let config = glossary_config(
            temp_path("missing"),
            std::env::temp_dir().join("petit-trad-embedding-model"),
            4,
        );

        let err = GlossaryStore::from_config(&config).expect_err("missing file should fail");
        assert!(matches!(
            err,
            Error::GlossaryRead(message) if message == format!("missing glossary file: {}", config.path.display())
        ));
    }

    #[test]
    fn glossary_from_config_reports_parse_failure() {
        let path = temp_path("parse-failure");
        fs::write(
            &path,
            "\
source_lang\ttarget_lang\tsource_term\n\
en\tfr\taccount balance\n",
        )
        .expect("test glossary file should be writable");
        let config = glossary_config(
            path.clone(),
            std::env::temp_dir().join("petit-trad-embedding-model"),
            4,
        );

        let err = GlossaryStore::from_config(&config).expect_err("parse failure should surface");
        assert!(matches!(
            err,
            Error::GlossaryParse(message) if message == "missing required header: target_term"
        ));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn glossary_from_config_reports_embedding_initialization_failure() {
        let path = temp_path("init-failure");
        fs::write(
            &path,
            "\
source_lang\ttarget_lang\tsource_term\ttarget_term\n\
            en\tfr\taccount balance\tsolde du compte\n",
        )
        .expect("test glossary file should be writable");
        let model_dir = temp_path("embedding-model-file");
        fs::write(&model_dir, "not a directory").expect("test model path should be writable");
        let config = glossary_config(path.clone(), model_dir.clone(), 4);

        let err =
            GlossaryStore::from_config(&config).expect_err("embedding init failure should surface");
        assert!(matches!(
            err,
            Error::GlossaryEmbeddingInit(message)
                if message == format!("embedding model directory is not a directory: {}", model_dir.display())
        ));

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(model_dir);
    }

    #[test]
    fn glossary_from_config_reports_missing_embedding_model_files() {
        let path = temp_path("missing-embedding-model");
        fs::write(
            &path,
            "\
source_lang\ttarget_lang\tsource_term\ttarget_term\n\
en\tfr\taccount balance\tsolde du compte\n",
        )
        .expect("test glossary file should be writable");
        let model_dir = std::env::temp_dir().join(format!(
            "petit-trad-empty-embedding-model-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&model_dir).expect("test model dir should be writable");
        let config = glossary_config(path.clone(), model_dir.clone(), 4);

        let err = GlossaryStore::from_config(&config)
            .expect_err("missing embedding model files should surface");
        assert!(matches!(
            err,
            Error::GlossaryEmbeddingInit(message)
                if message.contains(EMBEDDING_MODEL_CODE)
                    && message.contains(&model_dir.join(EMBEDDING_MODEL_FILE).display().to_string())
        ));

        let _ = fs::remove_file(path);
        let _ = fs::remove_dir_all(model_dir);
    }

    #[test]
    fn glossary_from_rows_reports_embedding_generation_failure() {
        let rows = vec![GlossaryRow {
            source_lang: "en".into(),
            target_lang: "fr".into(),
            source_term: "account balance".into(),
            target_term: "solde du compte".into(),
            note: None,
            source_term_norm: normalize_source_text("account balance"),
        }];
        let provider = Arc::new(StubEmbeddingProvider::failing_on_query(
            "query embedding unavailable",
        ));

        let err = build_store_from_rows(rows, provider, 4)
            .expect_err("embedding generation failure should surface");
        assert!(matches!(
            err,
            Error::GlossaryEmbeddingGenerate(message) if message == "query embedding unavailable"
        ));
    }

    #[test]
    fn glossary_from_rows_reports_index_build_failure() {
        let rows = vec![
            GlossaryRow {
                source_lang: "en".into(),
                target_lang: "fr".into(),
                source_term: "account balance".into(),
                target_term: "solde du compte".into(),
                note: None,
                source_term_norm: normalize_source_text("account balance"),
            },
            GlossaryRow {
                source_lang: "en".into(),
                target_lang: "fr".into(),
                source_term: "bank account".into(),
                target_term: "compte bancaire".into(),
                note: None,
                source_term_norm: normalize_source_text("bank account"),
            },
        ];
        let provider = Arc::new(StubEmbeddingProvider::deterministic(
            &[("account balance", &[1.0, 0.0]), ("bank account", &[1.0])],
            &[1.0, 0.0],
        ));

        let err = build_store_from_rows(rows, provider, 4).expect_err("index build should fail");
        assert!(matches!(
            err,
            Error::GlossaryIndexBuild(message) if message == "embedding dimensions mismatch"
        ));
    }
}
