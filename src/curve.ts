import { BN } from "@coral-xyz/anchor";
import { safeBN, unsafeBN, unsafeBnToNumber } from "@solocker/safe-bn";

export enum TradeDirection {
  AtoB = 0,
  BtoA = 1,
}

export abstract class CurveCalculator {
  abstract calculateInitialPrice(): number;
  static calculateAmountOut(
    initialPrice: number,
    amount: BN,
    tradeDirection: TradeDirection
  ): BN {
    throw new Error("method not implemented");
  }
}

export class ConstantCurveCalculator implements CurveCalculator {
  constructor(
    private supply: BN,
    private maximumTokenBReserveBalance: BN,
    private liquidityPercentage: number
  ) {}

  get tokenBReserveBalance() {
    return this.maximumTokenBReserveBalance
      .mul(new BN(this.liquidityPercentage))
      .div(new BN(100));
  }

  get boundingCurveSupply() {
    return this.supply.mul(new BN(this.liquidityPercentage)).div(new BN(100));
  }

  calculateInitialPrice(): number {
    const supply = this.boundingCurveSupply;
    const tokenBReserveBalance = this.tokenBReserveBalance;

    return unsafeBnToNumber(safeBN(tokenBReserveBalance).div(supply));
  }

  static calculateAmountOut(
    initialPrice: number,
    amount: BN,
    tradeDirection: TradeDirection
  ): BN {
    switch (tradeDirection) {
      case TradeDirection.AtoB:
        return unsafeBN(safeBN(initialPrice).mul(amount));
      case TradeDirection.BtoA:
        return unsafeBN(amount.div(safeBN(initialPrice)));
    }
  }
}
