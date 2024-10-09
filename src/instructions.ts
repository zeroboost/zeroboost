import { Program, web3 } from "@coral-xyz/anchor";
import {
  createInitializeMintInstruction,
  getAssociatedTokenAddressSync,
  MINT_SIZE,
  MintLayout,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  generatePubKey,
  getCreatePoolKeys,
  getPdaAmmConfigId,
} from "@raydium-io/raydium-sdk-v2";

import { publicKey } from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  findMetadataPda,
  MPL_TOKEN_METADATA_PROGRAM_ID,
} from "@metaplex-foundation/mpl-token-metadata";

import type { Zeroboost } from "../target/types/zeroboost";

import { devnet } from ".";
import {
  getBoundingCurveConfig,
  getBoundingCurvePda,
  getConfigPda,
  getMintPda,
  getPositionPda,
} from "./pda";

export const initializeConfig = (
  program: Program<Zeroboost>,
  admin: web3.PublicKey,
  params: Parameters<(typeof program)["methods"]["initializeConfig"]>[number],
  programId = devnet.ZERO_BOOST_PROGRAM
) => {
  const [config] = getConfigPda(programId);
  return program.methods.initializeConfig(params).accounts({ config, admin });
};

export const mintToken = (
  program: Program<Zeroboost>,
  pair: web3.PublicKey,
  creator: web3.PublicKey,
  params: Parameters<(typeof program)["methods"]["mintToken"]>[number],
  pythPairUsdFeed: web3.PublicKey,
  tokenMetadataProgram = MPL_TOKEN_METADATA_PROGRAM_ID,
  metadataFeeReciever = devnet.ZERO_BOOST_METADATA_FEE_RECIEVER
) => {
  const programId = program.programId;
  const [config] = getConfigPda(programId);
  const [mint] = getMintPda(params.name, params.symbol, creator, programId);
  const [metadata] = findMetadataPda(createUmi(program.provider.connection), {
    mint: publicKey(mint),
  });
  const {
    boundingCurve,
    boundingCurveAta,
    boundingCurveReserve,
    boundingCurveReserveAta,
    boundingCurveReservePairAta,
  } = getBoundingCurveConfig(mint, pair, programId);

  return program.methods.mintToken(params).accounts({
    mint,
    pair,
    config,
    creator,
    metadata,
    pythPairUsdFeed,
    boundingCurve,
    boundingCurveAta,
    boundingCurveReserve,
    boundingCurveReserveAta,
    boundingCurveReservePairAta,
    metadataFeeReciever,
    tokenMetadataProgram,
  });
};

export const buy = async (
  program: Program<Zeroboost>,
  mint: web3.PublicKey,
  payer: web3.PublicKey,
  params: Parameters<(typeof program)["methods"]["buy"]>[number]
) => {
  const programId = program.programId;
  const [boundingCurve] = getBoundingCurvePda(mint, programId);
  const {
    pair: { mint: pair },
  } = await program.account.boundingCurve.fetch(boundingCurve);
  const {
    boundingCurveReserve,
    boundingCurveReserveAta,
    boundingCurveReservePairAta,
  } = getBoundingCurveConfig(mint, pair, programId);

  const { publicKey: token, seed: tokenSeed } = generatePubKey({
    fromPublicKey: payer,
    programId: TOKEN_PROGRAM_ID,
  });
  const [position] = getPositionPda(token, programId);
  const payerTokenAta = getAssociatedTokenAddressSync(token, payer);
  const payerPairAta = getAssociatedTokenAddressSync(pair, payer);

  const mintRent =
    await program.provider.connection.getMinimumBalanceForRentExemption(
      MINT_SIZE
    );

  const mintInfo = MintLayout.decode(
    (await program.provider.connection.getAccountInfo(mint))!.data
  );

  return program.methods
    .buy(params)
    .preInstructions([
      web3.SystemProgram.createAccountWithSeed({
        fromPubkey: payer,
        newAccountPubkey: token,
        basePubkey: payer,
        seed: tokenSeed,
        lamports: mintRent,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeMintInstruction(
        token,
        mintInfo.decimals,
        boundingCurveReserve,
        boundingCurveReserve
      ),
    ])
    .accounts({
      token,
      mint,
      pair,
      payer,
      payerTokenAta,
      payerPairAta,
      position,
      boundingCurve,
      boundingCurveReserve,
      boundingCurveReserveAta,
      boundingCurveReservePairAta,
    });
};

export const sell = async (
  program: Program<Zeroboost>,
  mint: web3.PublicKey,
  token: web3.PublicKey,
  payer: web3.PublicKey,
  params: Parameters<(typeof program)["methods"]["sell"]>[number]
) => {
  const programId = program.programId;

  const [boundingCurve] = getBoundingCurvePda(mint, programId);
  const {
    pair: { mint: pair },
  } = await program.account.boundingCurve.fetch(boundingCurve);
  const {
    boundingCurveReserve,
    boundingCurveReserveAta,
    boundingCurveReservePairAta,
  } = getBoundingCurveConfig(mint, pair, programId);

  const [position] = getPositionPda(token, programId);
  const payerAta = getAssociatedTokenAddressSync(mint, payer);
  const payerTokenAta = getAssociatedTokenAddressSync(token, payer);
  const payerPairAta = getAssociatedTokenAddressSync(pair, payer);

  return program.methods.sell(params).accounts({
    mint,
    pair,
    token,
    payer,
    payerAta,
    payerTokenAta,
    payerPairAta,
    position,
    boundingCurve,
    boundingCurveReserve,
    boundingCurveReserveAta,
    boundingCurveReservePairAta,
  });
};

export const migrateFund = async (
  program: Program<Zeroboost>,
  boundingCurve: web3.PublicKey,
  payer: web3.PublicKey,
  params: Parameters<(typeof program)["methods"]["migrateFund"]>[number],
  raydiumCpPoolProgram = devnet.RAYDIUM_CP_POOL_PROGRAM,
  raydiumCpPoolFeeReciever = devnet.RAYDIUM_CP_FEE_RECIEVER
) => {
  const programId = program.programId;

  const [config] = getConfigPda(programId);
  const { mint, pair } = await program.account.boundingCurve.fetch(
    boundingCurve
  );
  const payerPairAta = getAssociatedTokenAddressSync(pair.mint, payer);
  const {
    boundingCurveAta,
    boundingCurveReserve,
    boundingCurveReserveAta,
    boundingCurveReservePairAta,
  } = getBoundingCurveConfig(mint, pair.mint, programId);
  const { publicKey: configId } = getPdaAmmConfigId(raydiumCpPoolProgram, 0);
  const poolkeys = getCreatePoolKeys({
    configId,
    mintA: pair.mint,
    mintB: mint,
    programId: raydiumCpPoolProgram,
  });

  const boundingCurveReserveLpAta = getAssociatedTokenAddressSync(
    poolkeys.lpMint,
    boundingCurveReserve,
    true
  );

  return program.methods.migrateFund(params).accounts({
    pair: pair.mint,
    mint,
    config,
    payer,
    payerPairAta,
    boundingCurve,
    boundingCurveAta,
    boundingCurveReserve,
    boundingCurveReserveAta,
    boundingCurveReservePairAta,
    boundingCurveReserveLpAta,
    ammConfig: poolkeys.configId,
    ammAuthority: poolkeys.authority,
    ammLpMint: poolkeys.lpMint,
    ammMintVault: poolkeys.vaultB,
    ammPairVault: poolkeys.vaultA,
    ammProgram: raydiumCpPoolProgram,
    ammFeeReceiver: raydiumCpPoolFeeReciever,
    ammPoolState: poolkeys.poolId,
    ammObservableState: poolkeys.observationId,
  });
};
