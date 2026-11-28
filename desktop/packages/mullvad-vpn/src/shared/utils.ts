export type NonEmptyArray<T> = [T, ...T[]];

export function hasValue<T>(value: T): value is NonNullable<T> {
  return value !== undefined && value !== null;
}

export function isInRanges(value: number, ranges: [number, number][]): boolean {
  return ranges.some(([min, max]) => value >= min && value <= max);
}

export function isNumber(number: unknown): number is number {
  return !Number.isNaN(number);
}
