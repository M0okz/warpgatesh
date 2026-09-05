import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { ModuleKind, transpileModule } from "typescript";

const source = await readFile(new URL("../src/state-refresh.ts", import.meta.url), "utf8");
const { outputText } = transpileModule(source, { compilerOptions: { module: ModuleKind.ESNext } });
const { createRefreshQueue } = await import(`data:text/javascript;base64,${Buffer.from(outputText).toString("base64")}`);

function deferred() {
  let resolve;
  let reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

test("a mutation during polling waits for fresh state and never publishes the old snapshot", async () => {
  const beforeMutation = deferred();
  const afterMutation = deferred();
  const published = [];
  let reads = 0;
  const refresh = createRefreshQueue(
    () => (++reads === 1 ? beforeMutation.promise : afterMutation.promise),
    (state) => published.push(state),
    assert.fail,
  );
  const poll = refresh();
  await Promise.resolve();
  const mutation = refresh(true);
  const anotherMutation = refresh(true);
  let mutationFinished = false;
  void mutation.then(() => { mutationFinished = true; });
  await Promise.resolve();
  assert.equal(mutationFinished, false, "the action must wait for the refresh after its change");
  beforeMutation.resolve("old preferences");
  await Promise.resolve();
  assert.deepEqual(published, [], "the in-flight snapshot predates the mutation");
  assert.equal(reads, 2, "concurrent changes share one trailing read");
  afterMutation.resolve("saved preferences");
  await Promise.all([poll, mutation, anotherMutation]);
  assert.deepEqual(published, ["saved preferences"]);
});

test("a failed read does not prevent the next refresh from recovering", async () => {
  let reads = 0;
  const errors = [];
  const published = [];
  const refresh = createRefreshQueue(
    async () => { if (++reads === 1) throw new Error("agent unavailable"); return "recovered"; },
    (value) => published.push(value),
    (error) => errors.push(error.message),
  );
  await refresh();
  await refresh();
  assert.deepEqual(errors, ["agent unavailable"]);
  assert.deepEqual(published, ["recovered"]);
});

test("polling during a slow read shares its result instead of starving publication", async () => {
  const read = deferred();
  const published = [];
  let reads = 0;
  const refresh = createRefreshQueue(
    () => { reads++; return read.promise; },
    (value) => published.push(value),
    assert.fail,
  );
  const first = refresh();
  await Promise.resolve();
  const second = refresh();
  const third = refresh();
  read.resolve("agent stopped");
  await Promise.all([first, second, third]);
  assert.equal(reads, 1);
  assert.deepEqual(published, ["agent stopped"]);
});
