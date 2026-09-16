import { runModel } from "./runner.mjs";
self.onmessage = async ({ data }) => {
  const result = await runModel(data.source, data.limit, stage => self.postMessage({ type: "stage", stage }));
  self.postMessage({ type: "result", ...result });
};
