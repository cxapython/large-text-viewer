use crate::file_reader::FileReader;
use grep_matcher::Matcher;
use grep_regex::RegexMatcherBuilder;
use grep_searcher::{Searcher, SearcherBuilder, Sink, SinkMatch};
use memchr::memmem;
use regex::Regex;
use std::io::Cursor;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::SyncSender,
    Arc,
};
use std::thread;

pub struct SearchEngine {
    query: String,
    use_regex: bool,
    case_sensitive: bool,
    regex: Option<Regex>,
    results: Vec<SearchResult>,
    total_results: usize,
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub byte_offset: usize,
    pub match_len: usize,
}

pub struct ChunkSearchResult {
    pub matches: Vec<SearchResult>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchType {
    Count,
    Fetch,
}

pub enum SearchMessage {
    ChunkResult(ChunkSearchResult),
    CountResult(usize),
    Done(SearchType),
    Error(String),
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            use_regex: false,
            case_sensitive: false,
            regex: None,
            results: Vec::new(),
            total_results: 0,
        }
    }

    pub fn set_query(&mut self, query: String, use_regex: bool, case_sensitive: bool) {
        self.query = query;
        self.use_regex = use_regex;
        self.case_sensitive = case_sensitive;

        let pattern = if use_regex {
            if !case_sensitive {
                format!("(?i){}", self.query)
            } else {
                self.query.clone()
            }
        } else if !case_sensitive {
            format!("(?i){}", regex::escape(&self.query))
        } else {
            regex::escape(&self.query)
        };

        self.regex = Regex::new(&pattern).ok();

        self.results.clear();
    }

    pub fn find_in_text(&self, text: &str) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();
        if self.query.is_empty() {
            return matches;
        }

        // 对于简单文本搜索，使用 memchr 的 memmem（SIMD 加速）
        if !self.use_regex {
            if self.case_sensitive {
                // 大小写敏感：直接使用 memmem
                let finder = memmem::Finder::new(self.query.as_bytes());
                let text_bytes = text.as_bytes();
                for start in finder.find_iter(text_bytes) {
                    matches.push((start, start + self.query.len()));
                }
            } else {
                // 大小写不敏感：需要转换后搜索，但保持原始位置
                // 使用 regex 引擎处理（它内部有优化）
                if let Some(re) = &self.regex {
                    for m in re.find_iter(text) {
                        matches.push((m.start(), m.end()));
                    }
                }
            }
        } else {
            // 正则表达式搜索
            if let Some(re) = &self.regex {
                for m in re.find_iter(text) {
                    matches.push((m.start(), m.end()));
                }
            }
        }
        matches
    }

    pub fn count_matches(
        &self,
        reader: Arc<FileReader>,
        tx: SyncSender<SearchMessage>,
        cancel_token: Arc<AtomicBool>,
    ) {
        let file_len = reader.len();
        if file_len == 0 || self.query.is_empty() {
            let _ = tx.send(SearchMessage::CountResult(0));
            let _ = tx.send(SearchMessage::Done(SearchType::Count));
            return;
        }

        let query = self.query.clone();
        let use_regex = self.use_regex;
        let case_sensitive = self.case_sensitive;

        thread::spawn(move || {
            // 使用 grep-searcher 进行计数
            let data = reader.all_data();

            // 构建 grep-regex 匹配器
            let pattern = if use_regex { query.clone() } else { regex::escape(&query) };
            let matcher = match RegexMatcherBuilder::new()
                .case_insensitive(!case_sensitive)
                .build(&pattern)
            {
                Ok(m) => m,
                Err(e) => {
                    let _ = tx.send(SearchMessage::Error(format!("Invalid regex: {}", e)));
                    return;
                }
            };

            // 对于简单文本搜索且大小写敏感，使用 memchr（最快）
            if !use_regex && case_sensitive {
                let finder = memmem::Finder::new(query.as_bytes());
                let count = finder.find_iter(data).count();
                let _ = tx.send(SearchMessage::CountResult(count));
                if !cancel_token.load(Ordering::Relaxed) {
                    let _ = tx.send(SearchMessage::Done(SearchType::Count));
                }
                return;
            }

            // 使用 grep-searcher 进行高性能搜索
            struct CountSink<'a> {
                count: usize,
                matcher: &'a grep_regex::RegexMatcher,
                cancel_token: Arc<AtomicBool>,
            }

            impl<'a> Sink for CountSink<'a> {
                type Error = std::io::Error;

                fn matched(
                    &mut self,
                    _searcher: &Searcher,
                    mat: &SinkMatch<'_>,
                ) -> Result<bool, Self::Error> {
                    if self.cancel_token.load(Ordering::Relaxed) {
                        return Ok(false);
                    }
                    // 计算这一行中的所有匹配
                    let line_bytes = mat.bytes();
                    let mut start = 0;
                    while start < line_bytes.len() {
                        match self.matcher.find_at(line_bytes, start) {
                            Ok(Some(m)) => {
                                self.count += 1;
                                start = m.end().max(start + 1);
                            }
                            _ => break,
                        }
                    }
                    Ok(true)
                }
            }

            let mut sink = CountSink {
                count: 0,
                matcher: &matcher,
                cancel_token: cancel_token.clone(),
            };

            let mut searcher = SearcherBuilder::new()
                .line_number(false)
                .build();

            let result = searcher.search_reader(&matcher, Cursor::new(data), &mut sink);

            match result {
                Ok(_) => {
                    let _ = tx.send(SearchMessage::CountResult(sink.count));
                    if !cancel_token.load(Ordering::Relaxed) {
                        let _ = tx.send(SearchMessage::Done(SearchType::Count));
                    }
                }
                Err(e) => {
                    let _ = tx.send(SearchMessage::Error(e.to_string()));
                }
            }
        });
    }

    pub fn fetch_matches(
        &self,
        reader: Arc<FileReader>,
        tx: SyncSender<SearchMessage>,
        start_offset: usize,
        max_results: usize,
        cancel_token: Arc<AtomicBool>,
    ) {
        let file_len = reader.len();
        if file_len == 0 || self.query.is_empty() {
            let _ = tx.send(SearchMessage::Done(SearchType::Fetch));
            return;
        }

        let query = self.query.clone();
        let use_regex = self.use_regex;
        let case_sensitive = self.case_sensitive;

        thread::spawn(move || {
            let data = reader.all_data();
            let search_data = if start_offset < data.len() {
                &data[start_offset..]
            } else {
                let _ = tx.send(SearchMessage::Done(SearchType::Fetch));
                return;
            };

            // 对于简单文本搜索且大小写敏感，使用 memchr（最快）
            if !use_regex && case_sensitive {
                let finder = memmem::Finder::new(query.as_bytes());
                let mut matches = Vec::new();
                let query_len = query.len();

                for pos in finder.find_iter(search_data) {
                    if cancel_token.load(Ordering::Relaxed) {
                        return;
                    }
                    matches.push(SearchResult {
                        byte_offset: start_offset + pos,
                        match_len: query_len,
                    });
                    if matches.len() >= max_results {
                        break;
                    }
                }

                if !matches.is_empty() {
                    let _ = tx.send(SearchMessage::ChunkResult(ChunkSearchResult { matches }));
                }
                if !cancel_token.load(Ordering::Relaxed) {
                    let _ = tx.send(SearchMessage::Done(SearchType::Fetch));
                }
                return;
            }

            // 构建 grep-regex 匹配器
            let pattern = if use_regex { query.clone() } else { regex::escape(&query) };
            let matcher = match RegexMatcherBuilder::new()
                .case_insensitive(!case_sensitive)
                .build(&pattern)
            {
                Ok(m) => m,
                Err(e) => {
                    let _ = tx.send(SearchMessage::Error(format!("Invalid regex: {}", e)));
                    return;
                }
            };

            // 收集匹配结果
            struct FetchSink<'a> {
                results: Vec<SearchResult>,
                max_results: usize,
                base_offset: usize,
                matcher: &'a grep_regex::RegexMatcher,
                cancel_token: Arc<AtomicBool>,
            }

            impl<'a> Sink for FetchSink<'a> {
                type Error = std::io::Error;

                fn matched(
                    &mut self,
                    _searcher: &Searcher,
                    mat: &SinkMatch<'_>,
                ) -> Result<bool, Self::Error> {
                    if self.cancel_token.load(Ordering::Relaxed) {
                        return Ok(false);
                    }

                    let line_bytes = mat.bytes();
                    let line_start_in_data = mat.absolute_byte_offset() as usize;

                    // 在这一行中找所有匹配
                    let mut start = 0;
                    while start < line_bytes.len() && self.results.len() < self.max_results {
                        match self.matcher.find_at(line_bytes, start) {
                            Ok(Some(m)) => {
                                self.results.push(SearchResult {
                                    byte_offset: self.base_offset + line_start_in_data + m.start(),
                                    match_len: m.end() - m.start(),
                                });
                                start = m.end().max(start + 1);
                            }
                            _ => break,
                        }
                    }

                    Ok(self.results.len() < self.max_results)
                }
            }

            let mut sink = FetchSink {
                results: Vec::new(),
                max_results,
                base_offset: start_offset,
                matcher: &matcher,
                cancel_token: cancel_token.clone(),
            };

            let mut searcher = SearcherBuilder::new()
                .line_number(false)
                .build();

            let result = searcher.search_reader(&matcher, Cursor::new(search_data), &mut sink);

            match result {
                Ok(_) => {
                    if !sink.results.is_empty() {
                        let _ = tx.send(SearchMessage::ChunkResult(ChunkSearchResult {
                            matches: sink.results,
                        }));
                    }
                    if !cancel_token.load(Ordering::Relaxed) {
                        let _ = tx.send(SearchMessage::Done(SearchType::Fetch));
                    }
                }
                Err(e) => {
                    let _ = tx.send(SearchMessage::Error(e.to_string()));
                }
            }
        });
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.results.clear();
        self.regex = None;
        self.total_results = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_reader::detect_encoding;
    use std::io::Write;
    use std::sync::mpsc;
    use tempfile::NamedTempFile;

    #[test]
    fn test_find_in_text() {
        let mut engine = SearchEngine::new();
        engine.set_query("test".to_string(), false, false);

        let text = "This is a test string. Another test.";
        let matches = engine.find_in_text(text);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (10, 14));
        assert_eq!(matches[1], (31, 35));
    }

    #[test]
    fn test_find_in_text_case_sensitive() {
        let mut engine = SearchEngine::new();
        engine.set_query("Test".to_string(), false, true);

        let text = "This is a Test string. Another test.";
        let matches = engine.find_in_text(text);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (10, 14));
    }

    #[test]
    fn test_find_in_text_memchr() {
        // 测试 memchr 快速路径
        let mut engine = SearchEngine::new();
        engine.set_query("error".to_string(), false, true); // case sensitive, non-regex

        let text = "error: something went wrong\nAnother error here\nerror again";
        let matches = engine.find_in_text(text);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_find_in_text_regex() {
        let mut engine = SearchEngine::new();
        engine.set_query(r"\d+".to_string(), true, false);

        let text = "There are 123 apples and 456 oranges.";
        let matches = engine.find_in_text(text);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (10, 13)); // "123"
        assert_eq!(matches[1], (25, 28)); // "456"
    }

    #[test]
    fn test_count_matches() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        write!(file, "test\ntest\ntest")?;
        let path = file.path().to_path_buf();

        let reader = Arc::new(FileReader::new(path, detect_encoding(b""))?);
        let mut engine = SearchEngine::new();
        engine.set_query("test".to_string(), false, false);

        let (tx, rx) = mpsc::sync_channel(10);
        let cancel_token = Arc::new(AtomicBool::new(false));

        engine.count_matches(reader, tx, cancel_token);

        let mut count = 0;
        loop {
            match rx.recv() {
                Ok(SearchMessage::CountResult(c)) => count += c,
                Ok(SearchMessage::Done(SearchType::Count)) => break,
                Ok(SearchMessage::Error(e)) => panic!("Error: {}", e),
                Ok(_) => continue,
                Err(_) => break,
            }
        }

        assert_eq!(count, 3);
        Ok(())
    }

    #[test]
    fn test_fetch_matches() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        write!(file, "error line1\nok line2\nerror line3")?;
        let path = file.path().to_path_buf();

        let reader = Arc::new(FileReader::new(path, detect_encoding(b""))?);
        let mut engine = SearchEngine::new();
        engine.set_query("error".to_string(), false, true); // case sensitive

        let (tx, rx) = mpsc::sync_channel(10);
        let cancel_token = Arc::new(AtomicBool::new(false));

        engine.fetch_matches(reader, tx, 0, 100, cancel_token);

        let mut results = Vec::new();
        loop {
            match rx.recv() {
                Ok(SearchMessage::ChunkResult(chunk)) => {
                    results.extend(chunk.matches);
                }
                Ok(SearchMessage::Done(SearchType::Fetch)) => break,
                Ok(SearchMessage::Error(e)) => panic!("Error: {}", e),
                Ok(_) => continue,
                Err(_) => break,
            }
        }

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].byte_offset, 0); // first "error"
        // "error line1\n" = 12 bytes, "ok line2\n" = 9 bytes, so second "error" starts at 21
        assert_eq!(results[1].byte_offset, 21); // second "error"
        Ok(())
    }

    #[test]
    fn test_memchr_fast_path() -> anyhow::Result<()> {
        // 测试 memchr 快速路径用于大小写敏感的纯文本搜索
        let mut file = NamedTempFile::new()?;
        let content = "hello world\nhello again\nworld hello";
        write!(file, "{}", content)?;
        let path = file.path().to_path_buf();

        let reader = Arc::new(FileReader::new(path, detect_encoding(b""))?);
        let mut engine = SearchEngine::new();
        engine.set_query("hello".to_string(), false, true);

        let (tx, rx) = mpsc::sync_channel(10);
        let cancel_token = Arc::new(AtomicBool::new(false));

        engine.count_matches(reader, tx, cancel_token);

        let mut count = 0;
        loop {
            match rx.recv() {
                Ok(SearchMessage::CountResult(c)) => count += c,
                Ok(SearchMessage::Done(SearchType::Count)) => break,
                Ok(_) => continue,
                Err(_) => break,
            }
        }

        assert_eq!(count, 3);
        Ok(())
    }
}
