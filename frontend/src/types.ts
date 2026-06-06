export interface Document {
  id: string;
  title: string;
  author: string;
  tags: string[];
  created_at: string;
  content: string;
  html: string;
}

export interface CreateDocumentRequest {
  title: string;
  author: string;
  tags: string[];
  content: string;
}

export interface UpdateDocumentRequest {
  title?: string;
  author?: string;
  tags?: string[];
  content?: string;
}

export interface SearchResult {
  document: Document;
  score: number;
  snippet: string;
}

export interface SearchResponse {
  total: number;
  results: SearchResult[];
  tokens: string[];
  took_ms: number;
}

export interface AutocompleteResult {
  query: string;
  count: number;
}

export interface SearchHistoryEntry {
  query: string;
  timestamp: string;
  ip?: string;
}

export interface BatchImportResult {
  success_count: number;
  fail_count: number;
  failures: BatchFailure[];
}

export interface BatchFailure {
  index: number;
  reason: string;
}

export interface ApiError {
  error: string;
  field?: string;
}
