extern crate cacache;
mod error;

pub struct Cache {
    path: String,
}

impl Cache {
    pub fn new(path: &str) -> Cache {
        Cache {
            path: path.to_string(),
        }
    }

    pub fn get<T: for<'a> serde::Deserialize<'a>>(&self, key: &'static str) -> error::Result<T> {
        let result = cacache::read_sync(&self.path, key);

        match result {
            Ok(data) => serde_json::from_slice(&data).map_err(|e| e.into()),
            Err(e) => {
                eprintln!("Error: {}", e);
                panic!("{}", e)
            }
        }
    }

    pub fn set<T: serde::Serialize>(&self, key: &str, value: &T) -> error::Result<()> {
        let result = serde_json::to_string(value);

        match result {
            Ok(data) => cacache::write_sync(&self.path, key, data.as_bytes())
                .map_err(|e| e.into())
                .map(|_| ()),
            Err(e) => {
                eprintln!("Error: {}", e);
                panic!("{}", e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct Foo {
        bar: String,
    }

    #[test]
    fn simple_keyval_cache() {
        let tmpf = std::env::temp_dir().join("test-cache");
        let cache = Cache::new(tmpf.to_str().unwrap());
        cache.set("foo", &"bar").unwrap();
        assert_eq!(cache.get::<String>("foo").unwrap(), "bar")
    }

    #[test]
    fn simple_key_struct_cache() {
        let tmpf = std::env::temp_dir().join("test-cache");
        let cache = Cache::new(tmpf.to_str().unwrap());
        cache
            .set(
                "foo1",
                &Foo {
                    bar: "baz".to_string(),
                },
            )
            .unwrap();
        assert_eq!(cache.get::<Foo>("foo1").unwrap().bar, "baz")
    }
}
