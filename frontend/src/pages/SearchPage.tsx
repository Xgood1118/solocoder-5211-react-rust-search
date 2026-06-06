import { useState, useEffect, useCallback, useRef } from 'react';
import { Link } from 'react-router-dom';
import DOMPurify from 'dompurify';
import { api } from '../api';
import type { SearchResponse } from '../types';

function SearchPage() {
  const [query, setQuery] = useState('');
  const [submittedQuery, setSubmittedQuery] = useState('');
  const [results, setResults] = useState<SearchResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [page, setPage] = useState(1);
  const [pageSize] = useState(20);
  const [showHistory, setShowHistory] = useState(false);
  const [showAutocomplete, setShowAutocomplete] = useState(false);
  const [history, setHistory] = useState<{ query: string; timestamp: string }[]>([]);
  const [autocompleteResults, setAutocompleteResults] = useState<{ query: string; count: number }[]>([]);
  const [searchMode, setSearchMode] = useState<'enter' | 'realtime'>('realtime');
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<number | null>(null);

  const loadHistory = useCallback(async () => {
    try {
      const data = await api.getHistory(20);
      setHistory(data);
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  const doSearch = useCallback(async (q: string, offset = 0) => {
    if (!q.trim()) return;

    setLoading(true);
    try {
      const data = await api.search(q, pageSize, offset);
      setResults(data);
      setSubmittedQuery(q);
    } catch (err) {
      console.error('搜索失败:', err);
    } finally {
      setLoading(false);
    }
  }, [pageSize]);

  const handleSearch = useCallback(() => {
    setPage(1);
    doSearch(query, 0);
    setShowHistory(false);
    setShowAutocomplete(false);
    loadHistory();
  }, [query, doSearch, loadHistory]);

  useEffect(() => {
    if (searchMode !== 'realtime') return;
    if (!query.trim()) {
      setResults(null);
      return;
    }

    if (debounceRef.current) {
      window.clearTimeout(debounceRef.current);
    }

    debounceRef.current = window.setTimeout(() => {
      setPage(1);
      doSearch(query, 0);
    }, 300);

    return () => {
      if (debounceRef.current) {
        window.clearTimeout(debounceRef.current);
      }
    };
  }, [query, searchMode, doSearch]);

  useEffect(() => {
    if (!query.trim()) {
      setAutocompleteResults([]);
      return;
    }

    const timer = window.setTimeout(async () => {
      try {
        const data = await api.autocomplete(query, 10);
        setAutocompleteResults(data);
      } catch {
        // ignore
      }
    }, 200);

    return () => window.clearTimeout(timer);
  }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (searchMode === 'enter') {
        handleSearch();
      }
      setShowHistory(false);
      setShowAutocomplete(false);
    }
  };

  const handleInputFocus = () => {
    setShowHistory(true);
  };

  const handleInputBlur = () => {
    setTimeout(() => {
      setShowHistory(false);
      setShowAutocomplete(false);
    }, 200);
  };

  const handleHistoryClick = (q: string) => {
    setQuery(q);
    setPage(1);
    doSearch(q, 0);
    setShowHistory(false);
  };

  const handleAutocompleteClick = (q: string) => {
    setQuery(q);
    setPage(1);
    doSearch(q, 0);
    setShowAutocomplete(false);
  };

  const deleteHistoryItem = async (q: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await api.deleteHistoryItem(q);
      setHistory(h => h.filter(item => item.query !== q));
    } catch (err) {
      console.error('删除失败:', err);
    }
  };

  const clearAllHistory = async () => {
    try {
      await api.clearHistory();
      setHistory([]);
    } catch (err) {
      console.error('清空失败:', err);
    }
  };

  const handlePageChange = (newPage: number) => {
    setPage(newPage);
    doSearch(submittedQuery, (newPage - 1) * pageSize);
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  const exportResults = () => {
    if (!results) return;
    const data = JSON.stringify(results, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `search-results-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const highlightText = (text: string, searchQuery: string) => {
    const tokens = results?.tokens || [];
    let result = text;

    const sortedTokens = [...new Set(tokens)].sort((a, b) => b.length - a.length);

    for (const token of sortedTokens) {
      if (!token.trim()) continue;
      const regex = new RegExp(`(${token.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
      result = result.replace(regex, '<mark class="highlight">$1</mark>');
    }

    return { __html: DOMPurify.sanitize(result) };
  };

  const totalPages = results ? Math.ceil(results.total / pageSize) : 0;

  return (
    <div className="search-page">
      <div className="search-header">
        <h1>知识库全文检索</h1>
        <p>快速找到你需要的技术文档、接口说明和会议纪要</p>
      </div>

      <div className="search-modes">
        <label>
          <input
            type="radio"
            name="searchMode"
            checked={searchMode === 'realtime'}
            onChange={() => setSearchMode('realtime')}
          />
          实时搜索（防抖 300ms）
        </label>
        <label>
          <input
            type="radio"
            name="searchMode"
            checked={searchMode === 'enter'}
            onChange={() => setSearchMode('enter')}
          />
          回车搜索
        </label>
      </div>

      <div className="search-box-container">
        <input
          ref={inputRef}
          type="text"
          className="search-box"
          placeholder="搜索文档、标签、作者... 支持 tag:xxx author:xxx after:yyyy-mm-dd"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          onFocus={handleInputFocus}
          onBlur={handleInputBlur}
        />
        <button className="search-btn" onClick={handleSearch}>搜索</button>

        {showAutocomplete && autocompleteResults.length > 0 && (
          <div className="autocomplete-dropdown">
            {autocompleteResults.map((item, idx) => (
              <div
                key={idx}
                className="autocomplete-item"
                onMouseDown={() => handleAutocompleteClick(item.query)}
              >
                <span>{item.query}</span>
                <span className="autocomplete-count">{item.count} 次</span>
              </div>
            ))}
          </div>
        )}

        {showHistory && history.length > 0 && query === '' && (
          <div className="search-history-dropdown">
            {history.map((item, idx) => (
              <div
                key={idx}
                className="history-item"
                onMouseDown={() => handleHistoryClick(item.query)}
              >
                <span>🕐 {item.query}</span>
                <button
                  className="history-delete"
                  onMouseDown={(e) => deleteHistoryItem(item.query, e)}
                >
                  删除
                </button>
              </div>
            ))}
            <div className="history-actions">
              <span style={{ fontSize: '13px', color: '#9ca3af' }}>搜索历史</span>
              <button className="clear-history-btn" onClick={clearAllHistory}>
                清空全部
              </button>
            </div>
          </div>
        )}
      </div>

      {results && results.tokens.length > 0 && (
        <div className="tokens-display">
          <span style={{ fontSize: '13px', color: '#6b7280', marginRight: '8px' }}>分词结果:</span>
          {results.tokens.map((token, idx) => (
            <span key={idx} className="token-tag">{token}</span>
          ))}
        </div>
      )}

      {results && (
        <div className="search-meta">
          <span>找到 {results.total} 条结果，耗时 {results.took_ms}ms</span>
          <button className="export-btn" onClick={exportResults}>
            导出 JSON
          </button>
        </div>
      )}

      {loading && <div className="loading">搜索中...</div>}

      {!loading && results && results.results.length === 0 && (
        <div className="empty-state">
          <h3>没有找到相关结果</h3>
          <p>试试其他关键词，或者检查一下拼写</p>
        </div>
      )}

      {!loading && results && results.results.length > 0 && (
        <div className="result-list">
          {results.results.map((result, idx) => (
            <div key={idx} className="result-item">
              <Link to={`/docs/${result.document.id}`} className="result-title">
                {result.document.title}
              </Link>
              <div className="result-meta">
                <span>作者: {result.document.author}</span>
                <span>{new Date(result.document.created_at).toLocaleDateString('zh-CN')}</span>
                <span className="result-score">相关度: {result.score.toFixed(2)}</span>
              </div>
              {result.document.tags.length > 0 && (
                <div className="result-tags" style={{ marginBottom: '10px' }}>
                  {result.document.tags.map((tag, i) => (
                    <span key={i} className="result-tag">{tag}</span>
                  ))}
                </div>
              )}
              <div
                className="result-snippet"
                dangerouslySetInnerHTML={highlightText(result.snippet, submittedQuery)}
              />
            </div>
          ))}
        </div>
      )}

      {!loading && results && totalPages > 1 && (
        <div className="pagination">
          <button
            onClick={() => handlePageChange(page - 1)}
            disabled={page === 1}
          >
            上一页
          </button>
          {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
            let pageNum = page - 2 + i;
            if (pageNum < 1) pageNum = i + 1;
            if (pageNum > totalPages) pageNum = totalPages - (4 - i);
            if (pageNum < 1) return null;
            return (
              <button
                key={pageNum}
                className={page === pageNum ? 'active' : ''}
                onClick={() => handlePageChange(pageNum)}
              >
                {pageNum}
              </button>
            );
          })}
          <button
            onClick={() => handlePageChange(page + 1)}
            disabled={page === totalPages}
          >
            下一页
          </button>
        </div>
      )}
    </div>
  );
}

export default SearchPage;
