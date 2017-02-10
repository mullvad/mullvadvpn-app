/**
 * Data incapsulation helper
 */
export default () => {
  const store = new WeakMap();
  return {
    get: (k) => store.get(k),
    set: (k, v) => store.set(k, v)
  };
};
