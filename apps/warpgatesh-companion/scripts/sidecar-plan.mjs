export function planSidecars(targetTriple, binaries) {
  const buildTargets =
    targetTriple === "universal-apple-darwin"
      ? ["aarch64-apple-darwin", "x86_64-apple-darwin"]
      : [targetTriple];

  return binaries.flatMap((binary) => {
    const architectureSidecars = buildTargets.map((buildTarget) => ({
      binary,
      destinationTarget: buildTarget,
      sourceTargets: [buildTarget],
    }));

    if (targetTriple !== "universal-apple-darwin") {
      return architectureSidecars;
    }

    return [
      ...architectureSidecars,
      {
        binary,
        destinationTarget: targetTriple,
        sourceTargets: buildTargets,
      },
    ];
  });
}
