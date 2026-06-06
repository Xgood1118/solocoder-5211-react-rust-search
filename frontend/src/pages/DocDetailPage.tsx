import { useState, useEffect } from 'react';
import { Link, useParams } from 'react-router-dom';
import DOMPurify from 'dompurify';
import { api } from '../api';
import type { Document } from '../types';

function DocDetailPage() {
  const { id } = useParams<{ id: string }>();
  const [doc, setDoc] = useState<Document | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!id) return;

    const loadDoc = async () => {
      try {
        const data = await api.getDoc(id);
        setDoc(data);
      } catch (err) {
        console.error('加载文档失败:', err);
      } finally {
        setLoading(false);
      }
    };

    loadDoc();
  }, [id]);

  if (loading) {
    return <div className="loading">加载中...</div>;
  }

  if (!doc) {
    return (
      <div className="doc-detail-page">
        <Link to="/docs" className="back-btn">← 返回列表</Link>
        <div className="empty-state">
          <h3>文档不存在</h3>
        </div>
      </div>
    );
  }

  const sanitizedHtml = DOMPurify.sanitize(doc.html);

  return (
    <div className="doc-detail-page">
      <Link to="/docs" className="back-btn">← 返回列表</Link>

      <div className="doc-detail-header">
        <h1>{doc.title}</h1>
        <div className="doc-detail-meta">
          <span>作者: {doc.author}</span>
          <span>创建于: {new Date(doc.created_at).toLocaleDateString('zh-CN')}</span>
        </div>
        {doc.tags.length > 0 && (
          <div className="doc-detail-tags">
            {doc.tags.map((tag, i) => (
              <span key={i} className="doc-tag">{tag}</span>
            ))}
          </div>
        )}
      </div>

      <div className="doc-content">
        <div
          className="markdown-body"
          dangerouslySetInnerHTML={{ __html: sanitizedHtml }}
        />
      </div>
    </div>
  );
}

export default DocDetailPage;
