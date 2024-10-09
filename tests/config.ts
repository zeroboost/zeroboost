import { web3, BN, Program } from "@coral-xyz/anchor";
import { Zeroboost } from "../target/types/zeroboost";

type ConfigParams = {
  metadataCreationFee: number;
  migrationPercentageFee: number;
  minimumCurveUsdValuation: number;
  maximumCurveUsdValuation: number;
  liquidityPercentage: number;
  mint: {
    name: string;
    symbol: string;
    uri: string;
    supply: number;
    decimals: number;
  };
};

export const buildConfig = (
  program: Program<Zeroboost>,
  {
    liquidityPercentage,
    mint: { name, symbol, uri, decimals, ...mintParams },
    ...params
  }: ConfigParams
) => {
  const SOL_USD_FEED = new web3.PublicKey(
    "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix"
  );

  const supply = BigInt(mintParams.supply) * BigInt(Math.pow(10, decimals));

  const [mint] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from(name), Buffer.from(symbol), program.provider.publicKey!.toBytes()],
    program.programId
  );

  const [boundingCurve] = web3.PublicKey.findProgramAddressSync(
    [mint.toBuffer(), Buffer.from("curve")],
    program.programId
  );

  return {
    name,
    symbol,
    uri,
    supply,
    decimals,
    mint,
    boundingCurve,
    SOL_USD_FEED,
    liquidityPercentage,
    ...params,
  };
};
