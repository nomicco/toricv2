import { assert, test } from "vitest";
import { runScenario, Scenario } from "@holochain/tryorama";
import { ActionHash } from "@holochain/client";
import { join } from "path";
import { fileURLToPath } from "url";
import { dirname } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const happPath = join(__dirname, "../../../../workdir/poi.happ");
const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));

async function createManifest(cell: any): Promise<ActionHash> {
  return cell.callZome({
    zome_name: "registry",
    fn_name: "create_manifest",
    payload: {
      blob: {
        blob_type: "ai_model",
        content_hash: "sha256:testmodel123",
        architecture: "llama",
        parameter_count: 7000000000,
        upstream_manifest_hashes: [],
        connector_source: "local",
        version: "1.0.0",
        description: "Test model for coordination",
      }
    }
  });
}

async function requestValidation(cell: any, manifestHash: ActionHash): Promise<ActionHash> {
  const metadata = new Uint8Array(0);
  return cell.callZome({
    zome_name: "coordination",
    fn_name: "request_validation",
    payload: {
      manifest_hash: manifestHash,
      metadata_blob: metadata,
    }
  });
}

async function submitEvaluation(
  cell: any,
  requestHash: ActionHash,
  passed: boolean,
  score: number
): Promise<ActionHash> {
  return cell.callZome({
    zome_name: "coordination",
    fn_name: "submit_evaluation",
    payload: {
      request_hash: requestHash,
      passed,
      score,
      details: passed ? "Model verified" : "Model failed verification",
    }
  });
}

test("request validation — creates a ValidationRequest entry", async () => {
  await runScenario(async (scenario: Scenario) => {
    const alice = await scenario.addPlayerWithApp({ appBundleSource: { type: "path", value: happPath } });
    await scenario.shareAllAgents();

    const manifestHash = await createManifest(alice.namedCells.get("registry")!);
    assert.ok(manifestHash);

    const requestHash = await requestValidation(
      alice.namedCells.get("coordination")!,
      manifestHash
    );
    assert.ok(requestHash);
  }, true, { disableLocalServices: true });
});

test("submit evaluation — validator can submit verdict", async () => {
  await runScenario(async (scenario: Scenario) => {
    const alice = await scenario.addPlayerWithApp({ appBundleSource: { type: "path", value: happPath } });
    await scenario.shareAllAgents();

    const manifestHash = await createManifest(alice.namedCells.get("registry")!);
    const requestHash = await requestValidation(
      alice.namedCells.get("coordination")!,
      manifestHash
    );

    const evalHash = await submitEvaluation(
      alice.namedCells.get("coordination")!,
      requestHash,
      true,
      0.95
    );
    assert.ok(evalHash);
  }, true, { disableLocalServices: true });
});

test("check quorum — below minimum validators returns not reached", async () => {
  await runScenario(async (scenario: Scenario) => {
    const alice = await scenario.addPlayerWithApp({ appBundleSource: { type: "path", value: happPath } });
    await scenario.shareAllAgents();

    const manifestHash = await createManifest(alice.namedCells.get("registry")!);
    const requestHash = await requestValidation(
      alice.namedCells.get("coordination")!,
      manifestHash
    );

    // Only one evaluation — below MIN_VALIDATORS of 3
    await submitEvaluation(
      alice.namedCells.get("coordination")!,
      requestHash,
      true,
      0.95
    );

    await sleep(500);

    const result = await alice.namedCells.get("coordination")!.callZome({
      zome_name: "coordination",
      fn_name: "check_quorum",
      payload: { request_hash: requestHash }
    });

    assert.equal(result.reached, false);
    assert.equal(result.evaluation_count, 1);
  }, true, { disableLocalServices: true });
});

test("check quorum — three validators agreeing reaches quorum", async () => {
  await runScenario(async (scenario: Scenario) => {
    const alice = await scenario.addPlayerWithApp({ appBundleSource: { type: "path", value: happPath } });
    const bob = await scenario.addPlayerWithApp({ appBundleSource: { type: "path", value: happPath } });
    const carol = await scenario.addPlayerWithApp({ appBundleSource: { type: "path", value: happPath } });
    await scenario.shareAllAgents();

    // Alice creates manifest and requests validation
    const manifestHash = await createManifest(alice.namedCells.get("registry")!);
    await sleep(2000);

    const requestHash = await requestValidation(
      alice.namedCells.get("coordination")!,
      manifestHash
    );
    await sleep(2000);

    // All three submit passing evaluations
    await submitEvaluation(alice.namedCells.get("coordination")!, requestHash, true, 0.95);
    await submitEvaluation(bob.namedCells.get("coordination")!, requestHash, true, 0.90);
    await submitEvaluation(carol.namedCells.get("coordination")!, requestHash, true, 0.88);

    await sleep(3000);

    // Alice checks quorum
    const result = await alice.namedCells.get("coordination")!.callZome({
      zome_name: "coordination",
      fn_name: "check_quorum",
      payload: { request_hash: requestHash }
    });

    assert.equal(result.reached, true);
    assert.equal(result.evaluation_count, 3);
    assert.ok(result.quorum_bundle_hash);
  }, true, { disableLocalServices: true });
});

test("get pending requests — agent can query their requests", async () => {
  await runScenario(async (scenario: Scenario) => {
    const alice = await scenario.addPlayerWithApp({ appBundleSource: { type: "path", value: happPath } });
    await scenario.shareAllAgents();

    const manifestHash = await createManifest(alice.namedCells.get("registry")!);
    await requestValidation(alice.namedCells.get("coordination")!, manifestHash);
    await requestValidation(alice.namedCells.get("coordination")!, manifestHash);

    await sleep(500);

    const requests = await alice.namedCells.get("coordination")!.callZome({
      zome_name: "coordination",
      fn_name: "get_pending_requests",
      payload: alice.agentPubKey
    });

    assert.equal(requests.length, 2);
  }, true, { disableLocalServices: true });
});
