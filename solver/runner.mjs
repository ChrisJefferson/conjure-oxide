import createConjure from "./conjure.mjs";

// Also used by the Node smoke tests, so they exercise the website's actual solver boundary.
export async function runModel(source, limit = 20, onStage = () => {}) {
  const started = performance.now();
  let diagnostics = "";
  const capture = line => { diagnostics = (diagnostics + line + "\n").slice(-16000); };
  try {
    onStage("loading");
    const engine = await createConjure({
      noInitialRun: true,
      locateFile: name => new URL(name, import.meta.url).href,
      print: capture,
      printErr: capture,
    });
    engine.FS.writeFile("/model.essence", source);
    onStage("solving");
    engine.callMain(["/model.essence", String(limit)]);
    const result = JSON.parse(engine.FS.readFile("/result.json", { encoding: "utf8" }));
    return { result, elapsedMs: performance.now() - started };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { result: { status: "error", error: diagnostics.trim() || message || "The solver stopped unexpectedly." }, elapsedMs: performance.now() - started };
  }
}
