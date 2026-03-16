use hdk::prelude::*;
use hdk::prelude::UnsafeBytes;

// ─────────────────────────────────────────────
// Blob type tags
// Every blob starts with a type field so the
// coordinator knows how to interpret it.
// ─────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "blob_type", rename_all = "snake_case")]
pub enum ManifestBlob {
    AiModel(AiModelManifest),
    Dataset(DatasetManifest),
    TrainingRun(TrainingRunManifest),
    InferenceEndpoint(InferenceEndpointManifest),
    Connector(ConnectorManifest),
    Generic(GenericManifest),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "blob_type", rename_all = "snake_case")]
pub enum AttestationBlob {
    ModelEvaluation(ModelEvaluationAttestation),
    DatasetAudit(DatasetAuditAttestation),
    ConnectorVerification(ConnectorVerificationAttestation),
    Generic(GenericAttestation),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "blob_type", rename_all = "snake_case")]
pub enum WarrantBlob {
    TamperedWeights(TamperedWeightsWarrant),
    MisrepresentedPerformance(MisrepresentedPerformanceWarrant),
    ConnectorMisbehavior(ConnectorMisbehaviorWarrant),
    FalseAttestation(FalseAttestationWarrant),
    Generic(GenericWarrant),
}

// ─────────────────────────────────────────────
// Manifest blobs
// ─────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiModelManifest {
    // Required
    pub content_hash: String,       // hash of the actual model weights
    pub architecture: String,       // e.g. "llama", "mistral", "gpt2"
    pub parameter_count: u64,       // number of parameters

    // Provenance chain
    pub upstream_manifest_hashes: Vec<ActionHash>, // training run, dataset, etc.
    pub connector_source: Option<String>,           // "bittensor", "gensyn", "akash", "local"

    // Optional metadata
    pub version: Option<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub artifact_timestamp: Option<u64>,
    pub fine_tuned_from: Option<String>,  // base model identifier
    pub training_data_description: Option<String>,
    pub quantization: Option<String>,     // e.g. "fp16", "int8", "gguf"
    pub context_length: Option<u64>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatasetManifest {
    pub content_hash: String,
    pub dataset_type: String,       // e.g. "instruction", "pretraining", "rlhf"
    pub record_count: Option<u64>,
    pub connector_source: Option<String>,
    pub upstream_manifest_hashes: Vec<ActionHash>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub artifact_timestamp: Option<u64>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrainingRunManifest {
    pub content_hash: String,
    pub model_manifest_hash: Option<ActionHash>,
    pub dataset_manifest_hash: Option<ActionHash>,
    pub connector_source: Option<String>,    // "gensyn", "bittensor", "local"
    pub compute_hours: Option<f64>,
    pub hardware: Option<String>,
    pub framework: Option<String>,           // "pytorch", "jax", etc.
    pub hyperparameters: Option<String>,     // JSON string, kept flexible
    pub artifact_timestamp: Option<u64>,
    pub upstream_manifest_hashes: Vec<ActionHash>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InferenceEndpointManifest {
    pub content_hash: String,
    pub model_manifest_hash: ActionHash,
    pub connector_source: Option<String>,    // "akash", "local", "runpod"
    pub endpoint_type: Option<String>,       // "http", "websocket", "grpc"
    pub artifact_timestamp: Option<u64>,
    pub upstream_manifest_hashes: Vec<ActionHash>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectorManifest {
    // Minimum required fields every connector must fill
    pub source_network_id: String,
    pub connector_version: String,
    pub content_hash: String,
    pub operator_pubkey: String,

    // Optional
    pub supported_manifest_types: Option<Vec<String>>,
    pub documentation_url: Option<String>,
    pub upstream_manifest_hashes: Vec<ActionHash>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericManifest {
    pub content_hash: String,
    pub manifest_type: String,
    pub upstream_manifest_hashes: Vec<ActionHash>,
    pub metadata: Option<String>,    // arbitrary JSON string
    pub artifact_timestamp: Option<u64>,
    pub tags: Option<Vec<String>>,
}

// ─────────────────────────────────────────────
// Attestation blobs
// ─────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelEvaluationAttestation {
    pub validation_method_hash: ActionHash,
    pub benchmark_type: String,      // e.g. "perplexity", "mmlu", "hellaswag", "custom"
    pub score: f64,
    pub passed: bool,
    pub confidence: Option<f64>,
    pub evaluation_details: Option<String>,  // JSON string for benchmark-specific data
    pub evaluated_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatasetAuditAttestation {
    pub validation_method_hash: ActionHash,
    pub audit_type: String,
    pub passed: bool,
    pub findings: Option<String>,
    pub evaluated_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectorVerificationAttestation {
    pub validation_method_hash: ActionHash,
    pub connector_manifest_hash: ActionHash,
    pub passed: bool,
    pub verified_schema_compliance: bool,
    pub verified_pipeline_compliance: bool,
    pub findings: Option<String>,
    pub evaluated_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericAttestation {
    pub validation_method_hash: ActionHash,
    pub attestation_type: String,
    pub passed: bool,
    pub score: Option<f64>,
    pub details: Option<String>,
    pub evaluated_at: Option<u64>,
}

// ─────────────────────────────────────────────
// Warrant blobs
// ─────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TamperedWeightsWarrant {
    pub severity: u8,               // 1-10
    pub evidence_hashes: Vec<ActionHash>,
    pub expected_hash: String,
    pub found_hash: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MisrepresentedPerformanceWarrant {
    pub severity: u8,
    pub evidence_hashes: Vec<ActionHash>,
    pub claimed_score: f64,
    pub actual_score: f64,
    pub benchmark_type: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectorMisbehaviorWarrant {
    pub severity: u8,
    pub evidence_hashes: Vec<ActionHash>,
    pub connector_manifest_hash: ActionHash,
    pub misbehavior_type: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FalseAttestationWarrant {
    pub severity: u8,
    pub evidence_hashes: Vec<ActionHash>,
    pub disputed_attestation_hash: ActionHash,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericWarrant {
    pub severity: u8,
    pub warrant_type: String,
    pub evidence_hashes: Vec<ActionHash>,
    pub description: Option<String>,
}

// ─────────────────────────────────────────────
// Serialization helpers
// ─────────────────────────────────────────────

pub fn encode_manifest_blob(blob: &ManifestBlob) -> ExternResult<SerializedBytes> {
    let bytes = serde_json::to_vec(blob).map_err(|e| {
        wasm_error!(WasmErrorInner::Guest(format!("Failed to serialize manifest blob: {}", e)))
    })?;
    Ok(SerializedBytes::from(UnsafeBytes::from(bytes)))
}

pub fn decode_manifest_blob(bytes: &SerializedBytes) -> ExternResult<ManifestBlob> {
    let raw: Vec<u8> = UnsafeBytes::from(bytes.clone()).into();
    serde_json::from_slice(&raw).map_err(|e| {
        wasm_error!(WasmErrorInner::Guest(format!("Failed to deserialize manifest blob: {}", e)))
    })
}

pub fn encode_attestation_blob(blob: &AttestationBlob) -> ExternResult<SerializedBytes> {
    let bytes = serde_json::to_vec(blob).map_err(|e| {
        wasm_error!(WasmErrorInner::Guest(format!("Failed to serialize attestation blob: {}", e)))
    })?;
    Ok(SerializedBytes::from(UnsafeBytes::from(bytes)))
}

pub fn decode_attestation_blob(bytes: &SerializedBytes) -> ExternResult<AttestationBlob> {
    let raw: Vec<u8> = UnsafeBytes::from(bytes.clone()).into();
    serde_json::from_slice(&raw).map_err(|e| {
        wasm_error!(WasmErrorInner::Guest(format!("Failed to deserialize attestation blob: {}", e)))
    })
}

pub fn encode_warrant_blob(blob: &WarrantBlob) -> ExternResult<SerializedBytes> {
    let bytes = serde_json::to_vec(blob).map_err(|e| {
        wasm_error!(WasmErrorInner::Guest(format!("Failed to serialize warrant blob: {}", e)))
    })?;
    Ok(SerializedBytes::from(UnsafeBytes::from(bytes)))
}

pub fn decode_warrant_blob(bytes: &SerializedBytes) -> ExternResult<WarrantBlob> {
    let raw: Vec<u8> = UnsafeBytes::from(bytes.clone()).into();
    serde_json::from_slice(&raw).map_err(|e| {
        wasm_error!(WasmErrorInner::Guest(format!("Failed to deserialize warrant blob: {}", e)))
    })
}