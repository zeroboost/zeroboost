import BN from "bn.js";
import { ConstantCurveCalculator, TradeDirection } from "../src";
import { expect } from "chai";

describe("Test constant curve js impl", () => {
  it("Should swap token out 1:1", () => {
    const supply = new BN(5_000_000).mul(new BN(10).pow(new BN(6)));
    const maximumTokenBBalanace = new BN(140).mul(new BN(10).pow(new BN(9)));

    const curve = new ConstantCurveCalculator(
      supply,
      maximumTokenBBalanace,
      25
    );

    const initialPrice = curve.calculateInitialPrice();
    const amountOut = ConstantCurveCalculator.calculateAmountOut(
      initialPrice,
      curve.tokenBReserveBalance,
      TradeDirection.BtoA
    );

    const pairAmountOut = ConstantCurveCalculator.calculateAmountOut(
      initialPrice,
      amountOut,
      TradeDirection.AtoB
    );
    
    expect(amountOut.eq(curve.boundingCurveSupply.div(new  BN(10).pow(new BN(6))))).equal(
      true,
      "Expect exert bounding curve supply"
    );
    expect(pairAmountOut.eq(curve.tokenBReserveBalance.div(new  BN(10).pow(new BN(6))))).equal(
      true,
      "Expect exert pair maxiumum reserve value"
    );
  });
});
