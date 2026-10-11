//! 번역 결과 캐시. 실시간 미리보기는 같은 문장을 여러 번 보내기 쉬워서, 무료 한도를 아끼려고 둔다.

use std::collections::{HashMap, VecDeque};

/// 오래된 것부터 버리는 단순한 크기 제한 캐시 (FIFO). 수백 개 규모라 LRU까지는 필요 없다.
pub struct TranslationCache<V> {
    capacity: usize,
    map: HashMap<(String, String), V>,
    order: VecDeque<(String, String)>,
}

impl<V: Clone> TranslationCache<V> {
    pub fn new(capacity: usize) -> Self {
        Self { capacity, map: HashMap::new(), order: VecDeque::new() }
    }

    pub fn get(&self, text: &str, target_lang: &str) -> Option<V> {
        self.map.get(&(text.to_owned(), target_lang.to_owned())).cloned()
    }

    pub fn insert(&mut self, text: &str, target_lang: &str, value: V) {
        let key = (text.to_owned(), target_lang.to_owned());
        if self.map.insert(key.clone(), value).is_none() {
            self.order.push_back(key);
        }
        while self.order.len() > self.capacity {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_includes_target_language() {
        let mut c = TranslationCache::new(10);
        c.insert("안녕", "EN-US", "Hello");
        assert_eq!(c.get("안녕", "EN-US"), Some("Hello"));
        assert_eq!(c.get("안녕", "JA"), None);
    }

    #[test]
    fn evicts_oldest_beyond_capacity() {
        let mut c = TranslationCache::new(2);
        c.insert("a", "EN-US", 1);
        c.insert("b", "EN-US", 2);
        c.insert("a", "EN-US", 10); // 같은 키 갱신은 순서를 늘리지 않는다
        c.insert("c", "EN-US", 3);
        assert_eq!(c.get("a", "EN-US"), None);
        assert_eq!(c.get("b", "EN-US"), Some(2));
        assert_eq!(c.get("c", "EN-US"), Some(3));
    }
}
