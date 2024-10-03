use super::{template::Error, Template};
use crate::config::get_config;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use tokio::sync::{Mutex, MutexGuard};

static TEMPLATES: Lazy<Mutex<Templates>> = Lazy::new(|| Mutex::new(Templates::new()));

pub struct Templates {
    templates: HashMap<PathBuf, Template>,
}

impl Templates {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub async fn get(&mut self, path: impl AsRef<Path> + Copy) -> Result<Template, Error> {
        let cache_templates = get_config().cache_templates;

        if let Some(t) = self.templates.get(path.as_ref()) {
            return Ok(t.clone());
        }

        let template = Template::new(path).await?;

        if cache_templates {
            self.templates.insert(path.as_ref().to_owned(), template);
            Ok(self.templates.get(path.as_ref()).unwrap().clone())
        } else {
            Ok(template)
        }
    }

    pub async fn cache() -> MutexGuard<'static, Templates> {
        TEMPLATES.lock().await
    }
}
