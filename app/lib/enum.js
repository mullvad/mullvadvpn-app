/**
 * Enum
 * @exports
 * @class Enum
 */
export default class Enum {

  /**
   * Creates an instance of EnumBase.
   * 
   * @param {...string|object} ... - enum keys
   * @memberOf Enum
   */
  constructor() {
    const items = [...arguments];
    let allKeys = [];
    let reverseMap = new Map();

    for(const item of items) {
      if(typeof(item) === 'string') {
        Object.defineProperty(this, item, {
          enumerable: true,
          value: item,
          writable: false
        });
        allKeys.push(item);
        reverseMap.set(item, item);
      } else if(typeof(item) === 'object') {
        for(const key of Object.keys(item)) {
          Object.defineProperty(this, key, {
            enumerable: true,
            value: item[key],
            writable: false
          });
          allKeys.push(key);
          reverseMap.set(item[key], key);
        }
      } else {
        throw new Error('Unsupported argument type: ' + typeof(item));
      }
    }

    Object.defineProperty(this, 'allKeys', {
      enumerable: false,
      value: allKeys,
      writable: false
    });
    
    Object.defineProperty(this, 'reverseMap', {
      enumerable: false,
      value: reverseMap,
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

  /**
   * Return key for value
   * 
   * @param {any} value
   * @returns {any|undefined} returns undefined if key is not found
   * 
   * @memberOf Enum
   */
  reverse(value) {
    return this.reverseMap.get(value);
  }
}

