import { useState, useEffect, useCallback } from 'react';
import { Link } from 'react-router-dom';
import { api } from '../api';
import type { Document, CreateDocumentRequest } from '../types';
import DocFormModal from '../components/DocFormModal';

function DocsPage() {
  const [docs, setDocs] = useState<Document[]>([]);
  const [loading, setLoading] = useState(false);
  const [tagFilter, setTagFilter] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState('');
  const [authorFilter, setAuthorFilter] = useState('');
  const [showModal, setShowModal] = useState(false);
  const [editingDoc, setEditingDoc] = useState<Document | null>(null);
  const [allTags, setAllTags] = useState<string[]>([]);
  const [allAuthors, setAllAuthors] = useState<string[]>([]);
  const [showTagSuggestions, setShowTagSuggestions] = useState(false);

  const loadDocs = useCallback(async () => {
    setLoading(true);
    try {
      const params: { tag?: string[]; author?: string } = {};
      if (tagFilter.length > 0) params.tag = tagFilter;
      if (authorFilter) params.author = authorFilter;
      const data = await api.listDocs(params);
      setDocs(data);

      const tags = new Set<string>();
      const authors = new Set<string>();
      data.forEach(doc => {
        doc.tags.forEach(t => tags.add(t));
        authors.add(doc.author);
      });
      setAllTags(Array.from(tags).sort());
      setAllAuthors(Array.from(authors).sort());
    } catch (err) {
      console.error('加载文档失败:', err);
    } finally {
      setLoading(false);
    }
  }, [tagFilter, authorFilter]);

  useEffect(() => {
    loadDocs();
  }, [loadDocs]);

  const handleCreate = () => {
    setEditingDoc(null);
    setShowModal(true);
  };

  const handleEdit = (doc: Document) => {
    setEditingDoc(doc);
    setShowModal(true);
  };

  const handleDelete = async (id: string) => {
    if (!confirm('确定要删除这篇文档吗？')) return;
    try {
      await api.deleteDoc(id);
      setDocs(d => d.filter(doc => doc.id !== id));
    } catch (err) {
      alert('删除失败');
    }
  };

  const handleSave = async (data: CreateDocumentRequest) => {
    try {
      if (editingDoc) {
        await api.updateDoc(editingDoc.id, data);
      } else {
        await api.createDoc(data);
      }
      setShowModal(false);
      setEditingDoc(null);
      loadDocs();
    } catch (err: any) {
      alert(err.message || '保存失败');
      throw err;
    }
  };

  const addTagFilter = (tag: string) => {
    if (tag && !tagFilter.includes(tag)) {
      setTagFilter([...tagFilter, tag]);
    }
    setTagInput('');
    setShowTagSuggestions(false);
  };

  const removeTagFilter = (tag: string) => {
    setTagFilter(tagFilter.filter(t => t !== tag));
  };

  const handleTagKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (tagInput.trim()) {
        addTagFilter(tagInput.trim());
      }
    }
  };

  const handleForceRebuild = async () => {
    if (!confirm('确定要强制重建索引吗？')) return;
    try {
      const result = await api.forceRebuild();
      alert(`索引重建成功，共 ${result.doc_count} 篇文档`);
    } catch (err) {
      alert('重建失败');
    }
  };

  const filteredTags = allTags.filter(
    t => t.toLowerCase().includes(tagInput.toLowerCase()) && !tagFilter.includes(t)
  );

  return (
    <div className="docs-page">
      <div className="docs-header">
        <h2>文档管理</h2>
        <div className="docs-actions">
          <button className="btn btn-secondary" onClick={handleForceRebuild}>
            重建索引
          </button>
          <button className="btn btn-primary" onClick={handleCreate}>
            + 新建文档
          </button>
        </div>
      </div>

      <div className="docs-filter">
        <div className="filter-group" style={{ minWidth: '300px' }}>
          <label>按标签筛选 (AND 语义)</label>
          <div className="tags-input-wrapper">
            {tagFilter.map(tag => (
              <span key={tag} className="tag-chip">
                {tag}
                <button onClick={() => removeTagFilter(tag)}>×</button>
              </span>
            ))}
            <input
              type="text"
              placeholder="输入标签后回车添加"
              value={tagInput}
              onChange={(e) => {
                setTagInput(e.target.value);
                setShowTagSuggestions(true);
              }}
              onKeyDown={handleTagKeyDown}
              onFocus={() => setShowTagSuggestions(true)}
              onBlur={() => setTimeout(() => setShowTagSuggestions(false), 200)}
            />
          </div>
          {showTagSuggestions && filteredTags.length > 0 && (
            <div className="tag-suggestions">
              {filteredTags.slice(0, 10).map(tag => (
                <div
                  key={tag}
                  className="tag-suggestion-item"
                  onMouseDown={() => addTagFilter(tag)}
                >
                  {tag}
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="filter-group">
          <label>按作者筛选</label>
          <select
            value={authorFilter}
            onChange={(e) => setAuthorFilter(e.target.value)}
            style={{
              padding: '8px 12px',
              border: '1px solid #e5e7eb',
              borderRadius: '6px',
              fontSize: '14px',
              background: '#fff',
              minWidth: '150px',
            }}
          >
            <option value="">全部作者</option>
            {allAuthors.map(author => (
              <option key={author} value={author}>{author}</option>
            ))}
          </select>
        </div>
      </div>

      {loading && <div className="loading">加载中...</div>}

      {!loading && docs.length === 0 && (
        <div className="empty-state">
          <h3>暂无文档</h3>
          <p>点击"新建文档"添加第一篇文档吧</p>
        </div>
      )}

      {!loading && docs.length > 0 && (
        <div className="doc-list">
          {docs.map(doc => (
            <div key={doc.id} className="doc-card">
              <div className="doc-info">
                <h3>
                  <Link to={`/docs/${doc.id}`}>{doc.title}</Link>
                </h3>
                <div className="doc-meta">
                  <span>作者: {doc.author}</span>
                  <span>{new Date(doc.created_at).toLocaleDateString('zh-CN')}</span>
                </div>
                {doc.tags.length > 0 && (
                  <div className="doc-tags">
                    {doc.tags.map((tag, i) => (
                      <span key={i} className="doc-tag">{tag}</span>
                    ))}
                  </div>
                )}
              </div>
              <div className="doc-card-actions">
                <button className="icon-btn" onClick={() => handleEdit(doc)}>
                  编辑
                </button>
                <button className="icon-btn delete" onClick={() => handleDelete(doc.id)}>
                  删除
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {showModal && (
        <DocFormModal
          initialData={editingDoc}
          onClose={() => {
            setShowModal(false);
            setEditingDoc(null);
          }}
          onSave={handleSave}
        />
      )}
    </div>
  );
}

export default DocsPage;
