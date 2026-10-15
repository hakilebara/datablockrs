import type {Block} from '../wasm-bindings/interfaces/datablockrs-blockstore-types.js'

function App({tree}: {tree: Block[]}) {
  return (
    <>
      <b>hello</b>
      <div>{tree.length}</div>
    </>
  )
}

export default App
