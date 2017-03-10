/**
 * Creates enum object with keys provided as arguments
 * 
 * @constructor
 * @type Enum
 * @property {bool} isValid({string}) whether key is valid
 * @param {...string} ...  Enum keys
 * @export
 * @returns Enum
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
