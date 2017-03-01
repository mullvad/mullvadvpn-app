/**
 * Creates enum object with keys provided as arguments
 */
export default function Enum() {
  let object = Object.create({});
  const keys = [...arguments];

  for(const key of keys) {
    Object.defineProperty(object, key, {
      enumerable: true,
      value: key,
      writable: false
    });
  }
  
  Object.defineProperty(object, 'isValid', {
    value: (e) => keys.includes(e)
  });

  return Object.freeze(object);
}
