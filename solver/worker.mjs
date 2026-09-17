import { runModel } from "./runner.mjs?v=minion-sat-1";
self.onmessage = async ({ data }) => {
  const result = await runModel(data.source, data.limit, stage => self.postMessage({ type: "stage", stage }), data.solver);
  self.postMessage({ type: "result", ...result });
};
