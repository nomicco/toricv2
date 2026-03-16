use hdi::prelude::*;

// ─────────────────────────────────────────────
// Entry Types
// Three generic entry types. All semantic meaning
// lives in the coordinator — blobs are opaque here.
// ─────────────────────────────────────────────

#[hdk_entry_helper]
#[derive(Clone)]
pub struct Manifest {
    pub metadata_blob: SerializedBytes,
}

#[hdk_entry_helper]
#[derive(Clone)]
pub struct Attestation {
    pub manifest_hash: ActionHash,
    pub metadata_blob: SerializedBytes,
}

#[hdk_entry_helper]
#[derive(Clone)]
pub struct Warrant {
    pub manifest_hash: ActionHash,
    pub metadata_blob: SerializedBytes,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[hdk_entry_types]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    Manifest(Manifest),
    Attestation(Attestation),
    Warrant(Warrant),
}

// ─────────────────────────────────────────────
// Link Types
// Append-only — delete always returns Invalid.
// ─────────────────────────────────────────────

#[hdk_link_types]
pub enum LinkTypes {
    AgentToManifest,
    ManifestToAttestation,
    ManifestToWarrant,
    AttestationToWarrant,
}

// ─────────────────────────────────────────────
// Genesis + Agent Joining
// ─────────────────────────────────────────────

#[hdk_extern]
pub fn genesis_self_check(
    _data: GenesisSelfCheckData,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_agent_joining(
    _agent_pub_key: AgentPubKey,
    _membrane_proof: &Option<MembraneProof>,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

// ─────────────────────────────────────────────
// Entry Validators
// ─────────────────────────────────────────────

fn validate_create_manifest(
    _action: Create,
    _manifest: Manifest,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

fn validate_create_attestation(
    _action: Create,
    attestation: Attestation,
) -> ExternResult<ValidateCallbackResult> {
    must_get_valid_record(attestation.manifest_hash)?;
    Ok(ValidateCallbackResult::Valid)
}

fn validate_create_warrant(
    _action: Create,
    warrant: Warrant,
) -> ExternResult<ValidateCallbackResult> {
    must_get_valid_record(warrant.manifest_hash)?;
    Ok(ValidateCallbackResult::Valid)
}

// ─────────────────────────────────────────────
// Link Validators
// ─────────────────────────────────────────────

fn validate_create_link_agent_to_manifest(
    _action: CreateLink,
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    AgentPubKey::try_from(base_address).map_err(|_| {
        wasm_error!(WasmErrorInner::Guest(
            "Base must be an AgentPubKey".to_string()
        ))
    })?;
    must_get_valid_record(
        ActionHash::try_from(target_address).map_err(|_| {
            wasm_error!(WasmErrorInner::Guest(
                "Target must be an ActionHash".to_string()
            ))
        })?,
    )?;
    Ok(ValidateCallbackResult::Valid)
}

fn validate_create_link_manifest_to_attestation(
    _action: CreateLink,
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    must_get_valid_record(
        ActionHash::try_from(base_address).map_err(|_| {
            wasm_error!(WasmErrorInner::Guest(
                "Base must be an ActionHash".to_string()
            ))
        })?,
    )?;
    must_get_valid_record(
        ActionHash::try_from(target_address).map_err(|_| {
            wasm_error!(WasmErrorInner::Guest(
                "Target must be an ActionHash".to_string()
            ))
        })?,
    )?;
    Ok(ValidateCallbackResult::Valid)
}

fn validate_create_link_manifest_to_warrant(
    _action: CreateLink,
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    must_get_valid_record(
        ActionHash::try_from(base_address).map_err(|_| {
            wasm_error!(WasmErrorInner::Guest(
                "Base must be an ActionHash".to_string()
            ))
        })?,
    )?;
    must_get_valid_record(
        ActionHash::try_from(target_address).map_err(|_| {
            wasm_error!(WasmErrorInner::Guest(
                "Target must be an ActionHash".to_string()
            ))
        })?,
    )?;
    Ok(ValidateCallbackResult::Valid)
}

fn validate_create_link_attestation_to_warrant(
    _action: CreateLink,
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    must_get_valid_record(
        ActionHash::try_from(base_address).map_err(|_| {
            wasm_error!(WasmErrorInner::Guest(
                "Base must be an ActionHash".to_string()
            ))
        })?,
    )?;
    must_get_valid_record(
        ActionHash::try_from(target_address).map_err(|_| {
            wasm_error!(WasmErrorInner::Guest(
                "Target must be an ActionHash".to_string()
            ))
        })?,
    )?;
    Ok(ValidateCallbackResult::Valid)
}

// ─────────────────────────────────────────────
// Validation Dispatcher
// ─────────────────────────────────────────────

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.flattened::<EntryTypes, LinkTypes>()? {

        FlatOp::StoreEntry(OpEntry::CreateEntry { app_entry, action }) => {
            match app_entry {
                EntryTypes::Manifest(manifest) =>
                    validate_create_manifest(action, manifest),
                EntryTypes::Attestation(attestation) =>
                    validate_create_attestation(action, attestation),
                EntryTypes::Warrant(warrant) =>
                    validate_create_warrant(action, warrant),
            }
        }

        FlatOp::StoreEntry(OpEntry::UpdateEntry { .. }) => {
            Ok(ValidateCallbackResult::Invalid(
                "Registry entries are immutable — updates are not permitted".to_string(),
            ))
        }

        FlatOp::RegisterUpdate(_) => {
            Ok(ValidateCallbackResult::Invalid(
                "Registry entries are immutable — updates are not permitted".to_string(),
            ))
        }

        FlatOp::RegisterDelete(_) => {
            Ok(ValidateCallbackResult::Invalid(
                "Registry entries are immutable — deletes are not permitted".to_string(),
            ))
        }

        FlatOp::RegisterCreateLink {
            link_type,
            base_address,
            target_address,
            tag,
            action,
        } => match link_type {
            LinkTypes::AgentToManifest =>
                validate_create_link_agent_to_manifest(action, base_address, target_address, tag),
            LinkTypes::ManifestToAttestation =>
                validate_create_link_manifest_to_attestation(action, base_address, target_address, tag),
            LinkTypes::ManifestToWarrant =>
                validate_create_link_manifest_to_warrant(action, base_address, target_address, tag),
            LinkTypes::AttestationToWarrant =>
                validate_create_link_attestation_to_warrant(action, base_address, target_address, tag),
        },

        FlatOp::RegisterDeleteLink { .. } => {
            Ok(ValidateCallbackResult::Invalid(
                "Registry links are permanent — deletes are not permitted".to_string(),
            ))
        }

        FlatOp::StoreRecord(OpRecord::CreateEntry { app_entry, action }) => {
            match app_entry {
                EntryTypes::Manifest(manifest) =>
                    validate_create_manifest(action, manifest),
                EntryTypes::Attestation(attestation) =>
                    validate_create_attestation(action, attestation),
                EntryTypes::Warrant(warrant) =>
                    validate_create_warrant(action, warrant),
            }
        }

        FlatOp::StoreRecord(OpRecord::UpdateEntry { .. }) => {
            Ok(ValidateCallbackResult::Invalid(
                "Registry entries are immutable — updates are not permitted".to_string(),
            ))
        }

        FlatOp::StoreRecord(OpRecord::DeleteEntry { .. }) => {
            Ok(ValidateCallbackResult::Invalid(
                "Registry entries are immutable — deletes are not permitted".to_string(),
            ))
        }

        FlatOp::StoreRecord(OpRecord::CreateLink {
            base_address,
            target_address,
            tag,
            link_type,
            action,
        }) => match link_type {
            LinkTypes::AgentToManifest =>
                validate_create_link_agent_to_manifest(action, base_address, target_address, tag),
            LinkTypes::ManifestToAttestation =>
                validate_create_link_manifest_to_attestation(action, base_address, target_address, tag),
            LinkTypes::ManifestToWarrant =>
                validate_create_link_manifest_to_warrant(action, base_address, target_address, tag),
            LinkTypes::AttestationToWarrant =>
                validate_create_link_attestation_to_warrant(action, base_address, target_address, tag),
        },

        FlatOp::StoreRecord(OpRecord::DeleteLink { .. }) => {
            Ok(ValidateCallbackResult::Invalid(
                "Registry links are permanent — deletes are not permitted".to_string(),
            ))
        }

        FlatOp::RegisterAgentActivity(OpActivity::CreateAgent { agent, action }) => {
            let previous_action = must_get_action(action.prev_action)?;
            match previous_action.action() {
                Action::AgentValidationPkg(AgentValidationPkg { membrane_proof, .. }) =>
                    validate_agent_joining(agent, membrane_proof),
                _ => Ok(ValidateCallbackResult::Invalid(
                    "The previous action for a `CreateAgent` action must be an `AgentValidationPkg`"
                        .to_string(),
                )),
            }
        }

        _ => Ok(ValidateCallbackResult::Valid),
    }
}