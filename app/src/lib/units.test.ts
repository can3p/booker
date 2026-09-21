import { describe, expect, it } from "vitest";

import { mm, toMm, toPixels } from "./units";

describe("lengths from the core", () => {
  it("convert between units", () => {
    expect(toMm("148mm")).toBeCloseTo(148);
    expect(toMm("1in")).toBeCloseTo(25.4);
    expect(toMm("72pt")).toBeCloseTo(25.4);
    expect(toPixels("1in")).toBeCloseTo(96);
  });

  it("refuse what is not a length", () => {
    expect(toMm("wide")).toBeNaN();
  });

  it("go back as millimetres the core can read", () => {
    expect(mm(12.3456)).toBe("12.35mm");
    expect(toMm(mm(40))).toBeCloseTo(40);
  });
});
