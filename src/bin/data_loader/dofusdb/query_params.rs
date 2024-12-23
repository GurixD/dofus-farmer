use std::collections::HashMap;

#[derive(Debug)]
pub struct QueryParams {
    inner: HashMap<String, Vec<String>>,
}

impl QueryParams {
    pub fn new() -> Self {
        let inner = HashMap::new();
        Self { inner }
    }

    pub fn set_param(&mut self, key: &str, value: &str) {
        self.inner.insert(key.to_string(), vec![value.to_string()]);
    }

    pub fn add_param(&mut self, key: &str, value: &str) {
        self.inner
            .entry(key.to_string())
            .or_default()
            .push(value.to_string());
    }

    pub fn remove_param(&mut self, key: &str) {
        self.inner.remove(key).unwrap();
    }

    pub fn to_query_string(&self) -> String {
        self.inner
            .iter()
            .filter_map(|(key, values)| {
                values
                    .iter()
                    .map(|value| key.clone() + "=" + value)
                    .reduce(|current, next| current + "&" + &next)
            })
            .reduce(|current, next| current + "&" + &next)
            .unwrap_or_default()
    }
}
