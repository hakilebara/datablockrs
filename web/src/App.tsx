import type {Block} from './api'
import './App.css'
import {types} from './api'
import {useState} from 'react'
import PageTitle from './PageTitle'

function App({root}: {root: Block}) {
  const [page, setPage] = useState<Block>(root)
  const [title, setTitle] = useState(root.type.tag === 'page' ? root.type.val.title : '')
  const [tree, setTree] = useState<[number, Block][]>(() => types.pageTree(root.id))

  function commitTitle(next: string) {
    if (page.type.tag !== 'page' || next === page.type.val.title) {
      setTitle(next)
      return
    }
    const updated = types.setBlock(page.id, {
      tag: 'page',
      val: {title: next, properties: page.type.val.properties},
    })
    setPage(updated)
    setTitle(next)
    types.appendChild({parentId: root.id, type: {tag: "text", val: {text: "hello"}}});
    setTree(types.pageTree(root.id));
  }

  function renderBlock(block: Block) {
    switch (block.type.tag) {
      case 'text':
        return <p
          contentEditable="plaintext-only"
          suppressContentEditableWarning={true}
          key={block.id}>{block.type.val.text}</p>
    }
  }

  return (
    <>
      <main className="layout">
        <div className="left"></div>
        <div className="top">
          <PageTitle value={title} onChange={setTitle} onCommit={commitTitle} />
        </div>
        <div className="content">
          {tree.map(([_depth, block]) => renderBlock(block))}
          <div className="clickzone">

          </div>
        </div>
        <div className="right"></div>
      </main>
    </>
  )
}

export default App
