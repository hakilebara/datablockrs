import {StrictMode} from 'react'
import {createRoot} from 'react-dom/client'
import './index.css'
import App from './App'
import {types} from './api'

types.initSchema();
let tree = types.childrenOf(undefined);

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App tree={tree} />
  </StrictMode>,
)
