export function planSidecars(targetTriple, binaries) {
  const buildTargets =
    targetTriple === "universal-apple-darwin"
      ? ["aarch64-apple-darwin", "x86_64-apple-darwin"]
      : [targetTriple];

  return binaries.flatMap((binary) =>
    buildTargets.map((buildTarget) => ({
      binary,
      destinationTarget: buildTarget,
      sourceTargets: [buildTarget],
    })),
  );
}
