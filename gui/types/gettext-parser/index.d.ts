declare module 'gettext-parser' {
  export namespace po {
    // eslint-disable-next-line @typescript-eslint/ban-types
    export function parse(input: string | Buffer, defaultCharset?: string): object;
  }
}
