// TypeScript typings for validated
// https://github.com/andreypopp/validated

declare module 'validated/schema' {
  export interface Context {}
  export interface ValidateResult<T> {
    context: Context;
    value: T;
  }

  export class Node<T> {
    validate(context: Context): ValidateResult<T>;
  }

  type NodeDict<T> = { [name: string]: Node<T> };

  export function object<T, S extends NodeDict<T>>(values: S, defaults?: Object): Node<S>;
  export function partialObject<T, S extends NodeDict<T>>(values: S, defaults?: Object): Node<S>;
  export function maybe<T>(valueNode: Node<T>): Node<T>;
  export const string: Node<string>;
  export const number: Node<number>;
  export const boolean: Node<boolean>;

  export function enumeration<T>(...values: Array<T>): Node<T>;
  export function arrayOf<T>(node: Node<T>): Node<T>;
  export function oneOf<A, B, C, D, E, F, G, H>(
    a: Node<A>,
    b?: Node<B>,
    c?: Node<C>,
    d?: Node<D>,
    e?: Node<E>,
    f?: Node<F>,
    g?: Node<G>,
    h?: Node<H>,
  ): Node<A | B | C | D | E | F | G | H>;
}

declare module 'validated/object' {
  export function validate<T>(object: Node<T>, value: any): T {
    return object.validate(new Context()).value;
  }
}
