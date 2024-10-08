import { BN } from "@coral-xyz/anchor";

export function getEstimatedRaydiumCpPoolCreationFee() {
  return new BN(2)
    .mul(new BN(10).pow(new BN(6)))
    .add(new BN(15).mul(new BN(10).pow(new BN(8))))
    .add(new BN(203938).mul(new BN(10).pow(new BN(1))));
}
