import {StrictMode} from 'react'
import {createRoot} from 'react-dom/client'
import './index.css'
import App from './App'
import {types, createPage} from './api'

types.initSchema();
const root = createPage("");

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App root={root} />
  </StrictMode>,
)
