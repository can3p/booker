/**
 * `Length` crosses from Rust as a string with its unit — `"148mm"`,
 * `"8.5in"`, `"12pt"` — so the window converts it here, in one place, rather
 * than every pane parsing lengths its own way.
 */

const PER_INCH = { mm: 25.4, cm: 2.54, in: 1, pt: 72, px: 96 } as const;

/** A length in millimetres, or `NaN` when it is not a length. */
export function toMm(length: string): number {
  const match = /^(-?[\d.]+)(mm|cm|in|pt|px)$/.exec(length.trim());
  if (!match) return Number.NaN;
  const unit = match[2] as keyof typeof PER_INCH;
  return (Number(match[1]) / PER_INCH[unit]) * PER_INCH.mm;
}

/** CSS pixels per millimetre at 100 %: a CSS pixel is 1/96 inch. */
export const PX_PER_MM = 96 / 25.4;

/** A length as CSS pixels at 100 %. */
export function toPixels(length: string): number {
  return toMm(length) * PX_PER_MM;
}

/** Millimetres as a `Length` the core accepts. */
export function mm(value: number): string {
  return `${Number(value.toFixed(2))}mm`;
}
