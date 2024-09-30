export type NonEmptyArray<T> = [T, ...T[]];

export function hasValue<T>(value: T): value is NonNullable<T> {
  return value !== undefined && value !== null;
}
