export interface SearchQuery {
  text: string;
  position: {
    request_url: boolean;
    request_header: boolean;
    request_body: boolean;
    response_header: boolean;
    response_body: boolean;
  };
}

export interface SearchItem {
  position: string;
  content: string;
  keyword_byte_index: number[];
}
export interface SearchData {
  id: string;
  url: string;
  search_item: SearchItem[];
}

export interface SearchResult {
  text: string;
  search_data: SearchData[];
}
