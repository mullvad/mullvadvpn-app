/**
 * Creates enum object with keys provided as arguments
 */
export default function Enum() {
  let object = {};

  for(const key of arguments) {
    Object.defineProperty(object, key, {
      enumerable: true,
      value: key,
      writable: false
    });
  }

  Object.freeze(object);

  return object;
}
