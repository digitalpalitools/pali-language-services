/* tslint:disable */
/* eslint-disable */
/**
* @param {string} str1
* @param {string} str2
* @returns {number}
*/
export function string_compare(str1: string, str2: string): number;
/**
* @param {string} str1
* @returns {number}
*/
export function string_length(str1: string): number;
/**
*/
export enum PaliAlphabet {
  A,
  AA,
  I,
  II,
  U,
  UU,
  E,
  O,
  K,
  KH,
  G,
  GH,
  QuoteN,
  C,
  CH,
  J,
  JH,
  TildeN,
  DotT,
  DotTH,
  DotD,
  DotDH,
  DotN,
  T,
  TH,
  D,
  DH,
  N,
  P,
  PH,
  B,
  BH,
  M,
  Y,
  R,
  L,
  V,
  S,
  H,
  DotL,
  DotM,
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly string_compare: (a: number, b: number, c: number, d: number) => number;
  readonly string_length: (a: number, b: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
        