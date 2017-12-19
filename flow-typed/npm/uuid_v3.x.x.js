// flow-typed signature: c07f382c8238bb78e545b60dd4f097a6
// flow-typed version: 27f92307d3/uuid_v3.x.x/flow_>=v0.33.x

declare module 'uuid' {
  declare function v1(options?: {|
    node?: number[],
    clockseq?: number,
    msecs?: number | Date,
    nsecs?: number,
  |}, buffer?: number[] | Buffer, offset?: number): string;
  declare function v4(options?: {|
    random?: number[],
    rng?: () => number[] | Buffer,
  |}, buffer?: number[] | Buffer, offset?: number): string;
}
