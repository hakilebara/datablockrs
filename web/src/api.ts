export {types} from '../wasm-bindings/datablockrs.js'
export type {Block, NewBlock, BlockType, Uuid} from "../wasm-bindings/interfaces/datablockrs-blockstore-types.js"

import {types} from '../wasm-bindings/datablockrs.js'
import type {Block} from "../wasm-bindings/interfaces/datablockrs-blockstore-types.js"

export function createPage(title: string): Block {
  return types.appendChild({
    parentId: undefined,
    type: {tag: 'page', val: {title, properties: []}}
  })
}
