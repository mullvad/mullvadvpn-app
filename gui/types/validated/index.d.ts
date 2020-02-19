// Type definitions for validated 2.0.1
// Project: https://github.com/andreypopp/validated
// Definitions by: Andrej Mihajlov <https://github.com/pronebird>
// TypeScript Version: 3.3.3

declare module 'validated/schema' {
  type Context = {};
  export interface ValidateResult<T> {
    context: Context;
    value: T;
  }

  // Cyclic refs issue when unnesting arrays with complex types
  // https://github.com/Microsoft/TypeScript/issues/14174
  type ExtractNodeType<T> = T extends Node<infer S>
    ? S extends { [name: string]: Node<infer _> }
      ? { [K in keyof S]: ExtractNodeType<S[K]> }
      : S extends Array<infer U>
      ? U[] // Array<ExtractNodeType<U>>
      : S
    : T;

  export class Node<T> {
    validate(context: Context): ValidateResult<ExtractNodeType<Node<T>>>;
  }

  type NodeDict = { [name: string]: Node<unknown> };

  // eslint-disable-next-line @typescript-eslint/ban-types
  export function object<T, S extends NodeDict>(values: S, defaults?: Object): Node<S>;
  // eslint-disable-next-line @typescript-eslint/ban-types
  export function partialObject<T, S extends NodeDict>(values: S, defaults?: Object): Node<S>;
  export function maybe<T>(valueNode: Node<T>): Node<T | undefined | null>;
  export const string: Node<string>;
  export const number: Node<number>;
  export const boolean: Node<boolean>;
  export function enumeration<T>(...values: Array<T>): Node<T>;
  export function arrayOf<T>(node: Node<T>): Node<Array<T>>;

  export function oneOf<A, B>(a: Node<A>, b: Node<B>): Node<A | B>;

  export function oneOf<A, B, C>(a: Node<A>, b: Node<B>, c: Node<C>): Node<A | B | C>;

  export function oneOf<A, B, C, D>(
    a: Node<A>,
    b: Node<B>,
    c: Node<C>,
    d: Node<D>,
  ): Node<A | B | C | D>;

  export function oneOf<A, B, C, D, E>(
    a: Node<A>,
    b: Node<B>,
    c: Node<C>,
    d: Node<D>,
    e: Node<E>,
  ): Node<A | B | C | D | E | F | G | H>;

  export function oneOf<A, B, C, D, E, F>(
    a: Node<A>,
    b: Node<B>,
    c: Node<C>,
    d: Node<D>,
    e: Node<E>,
    f: Node<F>,
  ): Node<A | B | C | D | E | F>;

  export function oneOf<A, B, C, D, E, F, G>(
    a: Node<A>,
    b: Node<B>,
    c: Node<C>,
    d: Node<D>,
    e: Node<E>,
    f: Node<F>,
    g: Node<G>,
  ): Node<A | B | C | D | E | F | G>;

  export function oneOf<A, B, C, D, E, F, G, H>(
    a: Node<A>,
    b: Node<B>,
    c: Node<C>,
    d: Node<D>,
    e: Node<E>,
    f: Node<F>,
    g: Node<G>,
    h: Node<H>,
  ): Node<A | B | C | D | E | F | G | H>;

  export function oneOf<A, B, C, D, E, F, G, H, I>(
    a: Node<A>,
    b: Node<B>,
    c: Node<C>,
    d: Node<D>,
    e: Node<E>,
    f: Node<F>,
    g: Node<G>,
    h: Node<H>,
    i: Node<I>,
  ): Node<A | B | C | D | E | F | G | H | I>;
}

declare module 'validated/object' {
  import { Node, ValidateResult } from 'validated/schema';

  type ExtractValidateResult<T> = ReturnType<T> extends ValidateResult<infer V> ? V : never;

  export function validate<T>(
    object: Node<T>,
    value: unknown,
  ): ExtractValidateResult<typeof object.validate>;
}
