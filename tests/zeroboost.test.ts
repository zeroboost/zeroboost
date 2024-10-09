import { expect } from "chai";

import { Program, web3 } from "@coral-xyz/anchor";
import { workspace, setProvider, AnchorProvider, BN } from "@coral-xyz/anchor";

import {
  createAssociatedTokenAccountIdempotent,
  createAssociatedTokenAccountIdempotentInstruction,
  createSyncNativeInstruction,
  getAssociatedTokenAddressSync,
  MintLayout,
  NATIVE_MINT,
} from "@solana/spl-token";
import { Amman } from "@metaplex-foundation/amman-client";

import {
  buy,
  getEstimatedRaydiumCpPoolCreationFee,
  initializeConfig,
  migrateFund,
  mintToken,
  sell,
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
      decimals: 6,
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
    const payerPairAta = getAssociatedTokenAddressSync(
      NATIVE_MINT,
      program.provider.publicKey!
    );

    const signature = await mintToken(
      program,
      NATIVE_MINT,
      program.provider.publicKey!,
      {
        name,
        symbol,
        uri,
        decimals,
        liquidityPercentage,
        isNative: true,
        supply: new BN(supply.toString()),
        migrationTarget: {
          raydium: {},
        },
      },
      SOL_USD_FEED
    )
      .postInstructions([
        createAssociatedTokenAccountIdempotentInstruction(
          program.provider.publicKey!,
          payerPairAta,
          program.provider.publicKey!,
          NATIVE_MINT
        ),
        web3.SystemProgram.transfer({
          fromPubkey: program.provider.publicKey!,
          toPubkey: payerPairAta,
          lamports: BigInt(1_000_000_000_000),
        }),
        createSyncNativeInstruction(payerPairAta),
      ])
      .rpc();

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

  it("Buy and sell minted token", async () => {
    const boundingCurveInfo = await program.account.boundingCurve.fetch(
      boundingCurve
    );

    const { pubkeys, signature } = await (
      await buy(program, boundingCurveInfo.mint, program.provider.publicKey!, {
        amount: boundingCurveInfo.maximumPairBalance,
      })
    ).rpcAndKeys();

    // const sellSignature = await (
    //   await sell(
    //     program,
    //     pubkeys.mint!,
    //     pubkeys.token!,
    //     program.provider.publicKey!,
    //     { amount: new BN(5_000).mul(new BN(10).pow(new BN(6))) }
    //   )
    // ).rpc();

    console.log("buy=", signature);
    // console.log("sell=", sellSignature);
  });

  it("Migrate fund", async () => {
    const signature = await (
      await migrateFund(program, boundingCurve, program.provider.publicKey!, {
        openTime: new BN(0),
      })
    )
      .preInstructions([
        web3.ComputeBudgetProgram.setComputeUnitLimit({
          units: 350_000,
        }),
      ])
      .rpc();

    console.log("migrate=", signature);
  });
});
