import { Routes, Route, Link, useLocation } from 'react-router-dom'
import SearchPage from './pages/SearchPage'
import DocsPage from './pages/DocsPage'
import DocDetailPage from './pages/DocDetailPage'

function App() {
  const location = useLocation()

  return (
    <div className="app">
      <header className="header">
        <div className="container">
          <div className="header-content">
            <Link to="/" className="logo">
              <span className="logo-icon">🔍</span>
              <span>知识库检索</span>
            </Link>
            <nav className="nav">
              <Link to="/" className={location.pathname === '/' ? 'nav-link active' : 'nav-link'}>
                搜索
              </Link>
              <Link to="/docs" className={location.pathname.startsWith('/docs') ? 'nav-link active' : 'nav-link'}>
                文档管理
              </Link>
            </nav>
          </div>
        </div>
      </header>

      <main className="main">
        <div className="container">
          <Routes>
            <Route path="/" element={<SearchPage />} />
            <Route path="/docs" element={<DocsPage />} />
            <Route path="/docs/:id" element={<DocDetailPage />} />
          </Routes>
        </div>
      </main>
    </div>
  )
}

export default App
