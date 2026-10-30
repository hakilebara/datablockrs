import type {Block} from './api'

function App({tree}: {tree: Block[]}) {
  return (
    <>
      <b>hello</b>
      <div>{tree.length}</div>
    </>
  )
}

export default App
