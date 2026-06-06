import { useState, useEffect } from 'react';
import type { Document, CreateDocumentRequest } from '../types';

interface DocFormModalProps {
  initialData: Document | null;
  onClose: () => void;
  onSave: (data: CreateDocumentRequest) => Promise<void>;
}

function DocFormModal({ initialData, onClose, onSave }: DocFormModalProps) {
  const [title, setTitle] = useState('');
  const [author, setAuthor] = useState('');
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState('');
  const [content, setContent] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (initialData) {
      setTitle(initialData.title);
      setAuthor(initialData.author);
      setTags(initialData.tags);
      setContent(initialData.content);
    }
  }, [initialData]);

  const addTag = () => {
    const tag = tagInput.trim();
    if (!tag) return;
    if (tags.includes(tag)) {
      setError('标签不能重复');
      return;
    }
    setTags([...tags, tag]);
    setTagInput('');
    setError(null);
  };

  const removeTag = (tag: string) => {
    setTags(tags.filter(t => t !== tag));
  };

  const handleTagKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      addTag();
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!title.trim()) {
      setError('标题不能为空');
      return;
    }
    if (title.length > 100) {
      setError('标题不能超过 100 字');
      return;
    }
    if (!author.trim()) {
      setError('作者不能为空');
      return;
    }
    if (!content.trim()) {
      setError('正文不能为空');
      return;
    }
    if (new Blob([content]).size > 1024 * 1024) {
      setError('正文不能超过 1MB');
      return;
    }

    setSubmitting(true);
    try {
      await onSave({
        title: title.trim(),
        author: author.trim(),
        tags,
        content,
      });
    } catch (err: any) {
      setError(err.message || '保存失败');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{initialData ? '编辑文档' : '新建文档'}</h3>
          <button className="modal-close" onClick={onClose}>×</button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>标题 *</label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="请输入文档标题"
              maxLength={100}
            />
          </div>

          <div className="form-group">
            <label>作者 *</label>
            <input
              type="text"
              value={author}
              onChange={(e) => setAuthor(e.target.value)}
              placeholder="请输入作者名"
            />
          </div>

          <div className="form-group">
            <label>标签</label>
            <div className="tags-input-wrapper">
              {tags.map(tag => (
                <span key={tag} className="tag-chip">
                  {tag}
                  <button type="button" onClick={() => removeTag(tag)}>×</button>
                </span>
              ))}
              <input
                type="text"
                value={tagInput}
                onChange={(e) => setTagInput(e.target.value)}
                onKeyDown={handleTagKeyDown}
                placeholder="输入标签后回车添加"
              />
            </div>
          </div>

          <div className="form-group">
            <label>正文 (Markdown) *</label>
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              placeholder="支持 Markdown 语法..."
              rows={15}
            />
          </div>

          {error && <div className="form-error">{error}</div>}

          <div className="modal-footer">
            <button
              type="button"
              className="btn btn-secondary"
              onClick={onClose}
              disabled={submitting}
            >
              取消
            </button>
            <button
              type="submit"
              className="btn btn-primary"
              disabled={submitting}
            >
              {submitting ? '保存中...' : '保存'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default DocFormModal;
