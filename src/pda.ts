import { web3 } from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { ZERO_BOOST_PROGRAM } from ".";

export const getConfigPda = (programId = ZERO_BOOST_PROGRAM) =>
  web3.PublicKey.findProgramAddressSync([Buffer.from("zeroboost")], programId);

export const getMintPda = (
  name: string,
  symbol: string,
  creator: web3.PublicKey,
  programId = ZERO_BOOST_PROGRAM
) => {
  const seeds = [name, symbol].map(Buffer.from);
  return web3.PublicKey.findProgramAddressSync([...seeds, creator.toBytes()], programId);
};

export const getBoundingCurvePda = (
  mint: web3.PublicKey,
  programId = ZERO_BOOST_PROGRAM
) => {
  const seeds = [mint.toBuffer(), Buffer.from("curve")];
  return web3.PublicKey.findProgramAddressSync(seeds, programId);
};

export const getBoundingCurveReservePda = (
  boundingCurve: web3.PublicKey,
  programId = ZERO_BOOST_PROGRAM
) => {
  const seeds = [boundingCurve.toBuffer(), Buffer.from("curve_reserve")];
  return web3.PublicKey.findProgramAddressSync(seeds, programId);
};

export const getBoundingCurveConfig = (
  mint: web3.PublicKey,
  pair: web3.PublicKey,
  programId = ZERO_BOOST_PROGRAM
) => {
  const [boundingCurve] = getBoundingCurvePda(mint, programId);
  const [boundingCurveReserve] = getBoundingCurveReservePda(
    boundingCurve,
    programId
  );
  const boundingCurveAta = getAssociatedTokenAddressSync(
    mint,
    boundingCurve,
    true
  );
  const boundingCurveReserveAta = getAssociatedTokenAddressSync(
    mint,
    boundingCurveReserve,
    true
  );
  const boundingCurveReservePairAta = getAssociatedTokenAddressSync(
    pair,
    boundingCurveReserve,
    true
  );

  return {
    boundingCurve,
    boundingCurveReserve,
    boundingCurveAta,
    boundingCurveReserveAta,
    boundingCurveReservePairAta,
  };
};

export const getCreatorConfig = (
  mint: web3.PublicKey,
  pair: web3.PublicKey
) => {};
