export type ObjectKeys<Value extends object> = Array<keyof Value>;

export function getObjectKeys<Value extends object>(object: Value): ObjectKeys<Value> {
  return Object.keys(object) as ObjectKeys<Value>;
}
