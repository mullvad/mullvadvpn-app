// eslint-disable-next-line @typescript-eslint/no-empty-function
const noop = () => {};

export default function consumePromise<T>(promise: Promise<T>): void {
  promise.then(noop, noop);
}
