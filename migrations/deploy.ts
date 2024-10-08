// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@coral-xyz/anchor");
import { AnchorProvider, Program } from "@coral-xyz/anchor";

import { IDL } from "../target/types/zeroboost";
import {
  devnet,
  getEstimatedRaydiumCpPoolCreationFee,
  initializeConfig,
} from "../src";

module.exports = async function (provider: AnchorProvider) {
  anchor.setProvider(provider);
  const program = new Program(IDL, devnet.ZERO_BOOST_PROGRAM, provider);

  const tx = await initializeConfig(program, program.provider.publicKey!, {
    metadataCreationFee: 1,
    migrationPercentageFee: 5,
    minimumCurveUsdValuation: 4000,
    maximumCurveUsdValuation: 60000,
    estimatedRaydiumCpPoolFee: getEstimatedRaydiumCpPoolCreationFee(),
  }).rpc();

  console.info("[info] zeroboost initialization signature=" + tx);
};
