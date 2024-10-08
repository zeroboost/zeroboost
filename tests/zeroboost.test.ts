import { expect } from "chai";

import { Program, web3 } from "@coral-xyz/anchor";
import { workspace, setProvider, AnchorProvider, BN } from "@coral-xyz/anchor";

import { MintLayout, NATIVE_MINT } from "@solana/spl-token";
import { Amman } from "@metaplex-foundation/amman-client";

import {
  getEstimatedRaydiumCpPoolCreationFee,
  initializeConfig,
  migrateFund,
  mintToken,
  swap,
} from "../src";
import { Zeroboost } from "../target/types/zeroboost";
import { buildConfig } from "./config";

export const amman = Amman.instance();

describe("zeroboost", async () => {
  setProvider(AnchorProvider.env());

  const program = workspace.Zeroboost as Program<Zeroboost>;

  const {
    metadataCreationFee,
    migrationPercentageFee,
    minimumCurveUsdValuation,
    maximumCurveUsdValuation,
    liquidityPercentage,
    name,
    supply,
    symbol,
    uri,
    SOL_USD_FEED,
    mint,
    boundingCurve,
    decimals,
  } = buildConfig(program, {
    metadataCreationFee: 5,
    migrationPercentageFee: 5,
    minimumCurveUsdValuation: 4000,
    maximumCurveUsdValuation: 60000,
    liquidityPercentage: 25,
    mint: {
      name: "FliedLice",
      symbol: "FLIEDLICE",
      uri: "https://fliedlice.xyz",
      supply: 1_000_000_000,
      decimals: 9,
    },
  });

  it("Initialize zeroboost config account", async () => {
    const { pubkeys, signature } = await initializeConfig(
      program,
      program.provider.publicKey!,
      {
        metadataCreationFee,
        migrationPercentageFee,
        minimumCurveUsdValuation,
        maximumCurveUsdValuation,
        estimatedRaydiumCpPoolFee: getEstimatedRaydiumCpPoolCreationFee(),
      }
    ).rpcAndKeys();

    console.log("config=", signature);

    const config = await program.account.config.fetch(pubkeys.config!);

    expect(config.metadataCreationFee).equals(
      metadataCreationFee,
      "Invalid metadata creation fee"
    );
    expect(config.migrationPercentageFee).equals(
      migrationPercentageFee,
      "Invalid migration percentage fee"
    );
    expect(config.maximumCurveUsdValuation).equals(
      maximumCurveUsdValuation,
      "Invalid  maximum curve usd valuation"
    );
    expect(config.minimumCurveUsdValuation).equals(
      minimumCurveUsdValuation,
      "Invalid minimum curve usd valuation"
    );
  });

  it("Create mint and curve info", async () => {
    const instructions = await mintToken(
      program,
      NATIVE_MINT,
      program.provider.publicKey!,
      {
        name,
        symbol,
        uri,
        decimals,
        liquidityPercentage,
        supply: new BN(supply.toString()),
        migrationTarget: {
          raydium: {},
        },
      },
      SOL_USD_FEED
    ).instruction();

    const transaction = new web3.Transaction()
      .add(
        web3.ComputeBudgetProgram.setComputeUnitLimit({
          units: 250_000,
        })
      )
      .add(instructions);
    const signature = await program.provider!.sendAndConfirm!(transaction);

    console.log("mint=", signature);

    const mintInfo = MintLayout.decode(
      Uint8Array.from(
        (await program.provider.connection.getAccountInfo!(mint))!.data
      )
    );

    let boundingCurveInfo = await program.account.boundingCurve.fetch(
      boundingCurve
    );

    expect(mintInfo.isInitialized).equal(true, "Mint uninitialize");
    expect(mintInfo.supply).equal(supply, "Invalid mint supply");
    expect(mintInfo.decimals).equal(decimals, "Invalid mint decimals");

    expect(boundingCurveInfo.migrated).equal(false, "must not be migrated");
    expect(boundingCurveInfo.liquidityPercentage).equal(
      liquidityPercentage,
      "Invalid liquidity percentage"
    );
  });

  it("Buy minted token", async () => {
    const boundingCurveInfo = await program.account.boundingCurve.fetch(
      boundingCurve
    );

    const signature = await (
      await swap(program, boundingCurveInfo.mint, program.provider.publicKey!, {
        amount: boundingCurveInfo.maximumPairBalance,
        tradeDirection: 0,
      })
    ).rpc();

    console.log("buy=", signature);
  });

  it("Migrate fund", async () => {
    const instructions = await (
      await migrateFund(program, boundingCurve, program.provider.publicKey!, {
        openTime: new BN(0),
      })
    ).instruction();

    const transaction = new web3.Transaction();
    transaction.add(
      web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 })
    );
    transaction.add(instructions);

    const tx = await program.provider.sendAndConfirm!(transaction);

    console.log("migrate=", tx);
  });
});
