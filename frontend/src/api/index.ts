import type {
  Document,
  CreateDocumentRequest,
  UpdateDocumentRequest,
  SearchResponse,
  AutocompleteResult,
  SearchHistoryEntry,
  BatchImportResult,
} from '../types';

const API_BASE = '/api';

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const err = await response.json().catch(() => ({ error: '请求失败' }));
    throw new Error(err.error || `HTTP ${response.status}`);
  }
  return response.json();
}

export const api = {
  search: (q: string, limit = 20, offset = 0): Promise<SearchResponse> => {
    const params = new URLSearchParams({ q, limit: String(limit), offset: String(offset) });
    return fetch(`${API_BASE}/search?${params}`).then(handleResponse<SearchResponse>);
  },

  autocomplete: (q: string, limit = 10): Promise<AutocompleteResult[]> => {
    const params = new URLSearchParams({ q, limit: String(limit) });
    return fetch(`${API_BASE}/autocomplete?${params}`).then(handleResponse<AutocompleteResult[]>);
  },

  listDocs: (params?: { tag?: string[]; author?: string; limit?: number; offset?: number }): Promise<Document[]> => {
    const usp = new URLSearchParams();
    if (params?.tag) params.tag.forEach(t => usp.append('tag', t));
    if (params?.author) usp.set('author', params.author);
    if (params?.limit) usp.set('limit', String(params.limit));
    if (params?.offset) usp.set('offset', String(params.offset));
    return fetch(`${API_BASE}/docs?${usp}`).then(handleResponse<Document[]>);
  },

  getDoc: (id: string): Promise<Document> => {
    return fetch(`${API_BASE}/docs/${id}`).then(handleResponse<Document>);
  },

  createDoc: (doc: CreateDocumentRequest): Promise<Document> => {
    return fetch(`${API_BASE}/docs`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(doc),
    }).then(handleResponse<Document>);
  },

  updateDoc: (id: string, doc: UpdateDocumentRequest): Promise<Document> => {
    return fetch(`${API_BASE}/docs/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(doc),
    }).then(handleResponse<Document>);
  },

  deleteDoc: (id: string): Promise<void> => {
    return fetch(`${API_BASE}/docs/${id}`, { method: 'DELETE' }).then(r => {
      if (!r.ok) throw new Error('删除失败');
    });
  },

  batchImport: (docs: CreateDocumentRequest[]): Promise<BatchImportResult> => {
    return fetch(`${API_BASE}/docs/batch`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(docs),
    }).then(handleResponse<BatchImportResult>);
  },

  forceRebuild: (): Promise<{ status: string; doc_count: number }> => {
    return fetch(`${API_BASE}/force-rebuild`, { method: 'POST' }).then(handleResponse<{ status: string; doc_count: number }>);
  },

  getHistory: (limit = 50): Promise<SearchHistoryEntry[]> => {
    const params = new URLSearchParams({ limit: String(limit) });
    return fetch(`${API_BASE}/history?${params}`).then(handleResponse<SearchHistoryEntry[]>);
  },

  deleteHistoryItem: (q: string): Promise<void> => {
    const params = new URLSearchParams({ q });
    return fetch(`${API_BASE}/history?${params}`, { method: 'DELETE' }).then(r => {
      if (!r.ok) throw new Error('删除失败');
    });
  },

  clearHistory: (): Promise<void> => {
    return fetch(`${API_BASE}/history/clear`, { method: 'POST' }).then(r => {
      if (!r.ok) throw new Error('清空失败');
    });
  },
};
