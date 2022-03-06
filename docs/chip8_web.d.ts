/* tslint:disable */
/* eslint-disable */
/**
*/
export function start(): void;
/**
*/
export class Chip8 {
  free(): void;
/**
* @returns {Chip8}
*/
  static new(): Chip8;
/**
* @param {Uint8Array} rom
*/
  load(rom: Uint8Array): void;
/**
* @param {number} key
* @param {boolean} pressed
*/
  set_pressed_key(key: number, pressed: boolean): void;
/**
*/
  tick(): void;
/**
* @returns {number}
*/
  display_data(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_chip8_free: (a: number) => void;
  readonly chip8_new: () => number;
  readonly chip8_load: (a: number, b: number, c: number, d: number) => void;
  readonly chip8_set_pressed_key: (a: number, b: number, c: number) => void;
  readonly chip8_tick: (a: number) => void;
  readonly chip8_display_data: (a: number) => number;
  readonly start: () => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_start: () => void;
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
