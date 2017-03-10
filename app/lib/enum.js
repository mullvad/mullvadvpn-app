/**
 * Enum
 * @exports
 * @class Enum
 */
export default class Enum {

  /**
   * Creates an instance of EnumBase.
   * 
   * @param {...string} ... - enum keys
   * @memberOf Enum
   */
  constructor() {
    const keys = [...arguments];

    for(const key of keys) {
      Object.defineProperty(this, key, {
        enumerable: true,
        value: key,
        writable: false
      });
    }

    Object.defineProperty(this, 'allKeys', {
      enumerable: false,
      value: keys,
      writable: false
    });

    Object.freeze(this);
  }

  /**
   * Check if key is registered in this enum
   * 
   * @param {string} key 
   * @returns {bool}
   * 
   * @memberOf Enum
   */
  isValid(key) { 
    return this.allKeys.includes(key); 
  }
}

