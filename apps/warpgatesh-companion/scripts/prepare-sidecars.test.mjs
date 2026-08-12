import assert from "node:assert/strict";
import test from "node:test";

import { planSidecars } from "./sidecar-plan.mjs";

test("prepares one sidecar per architecture for a universal macOS build", () => {
  const plan = planSidecars("universal-apple-darwin", [
    "warpgatesh",
    "warpgatesh-agent",
  ]);

  assert.deepEqual(
    plan.map(({ binary, destinationTarget, sourceTargets }) => ({
      binary,
      destinationTarget,
      sourceTargets,
    })),
    [
      {
        binary: "warpgatesh",
        destinationTarget: "aarch64-apple-darwin",
        sourceTargets: ["aarch64-apple-darwin"],
      },
      {
        binary: "warpgatesh",
        destinationTarget: "x86_64-apple-darwin",
        sourceTargets: ["x86_64-apple-darwin"],
      },
      {
        binary: "warpgatesh-agent",
        destinationTarget: "aarch64-apple-darwin",
        sourceTargets: ["aarch64-apple-darwin"],
      },
      {
        binary: "warpgatesh-agent",
        destinationTarget: "x86_64-apple-darwin",
        sourceTargets: ["x86_64-apple-darwin"],
      },
    ],
  );
});

test("keeps a native sidecar paired with its target", () => {
  assert.deepEqual(planSidecars("aarch64-apple-darwin", ["warpgatesh"]), [
    {
      binary: "warpgatesh",
      destinationTarget: "aarch64-apple-darwin",
      sourceTargets: ["aarch64-apple-darwin"],
    },
  ]);
});
