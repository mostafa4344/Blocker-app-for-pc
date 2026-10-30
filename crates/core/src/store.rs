//! JSON persistence with atomic writes.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Serialize};

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: impl Into<PathBuf>) -> io::Result<Store> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Store { dir })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// Loads `name`, falling back to the default when missing.
    /// A corrupt file is kept aside as `<name>.corrupt` rather than overwritten silently.
    pub fn load<T: DeserializeOwned + Default>(&self, name: &str) -> io::Result<T> {
        let p = self.path(name);
        match fs::read(&p) {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(v) => Ok(v),
                Err(_) => {
                    let _ = fs::rename(&p, self.path(&format!("{name}.corrupt")));
                    Ok(T::default())
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(T::default()),
            Err(e) => Err(e),
        }
    }

    pub fn save<T: Serialize>(&self, name: &str, value: &T) -> io::Result<()> {
        let tmp = self.path(&format!("{name}.tmp"));
        {
            // A leftover temp file may have been planted by another account:
            // remove it and create a fresh one (owned by us) instead of reusing it.
            let _ = fs::remove_file(&tmp);
            let mut f = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp)?;
            f.write_all(&serde_json::to_vec_pretty(value).map_err(io::Error::other)?)?;
            f.sync_all()?;
        }
        fs::rename(tmp, self.path(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Config;

    #[test]
    fn roundtrip_and_corruption() {
        let dir = std::env::temp_dir().join(format!("ff-store-{}", crate::new_id()));
        let s = Store::new(&dir).unwrap();
        let missing: Config = s.load("config.json").unwrap();
        assert_eq!(missing, Config::default());
        let mut c = Config::default();
        c.settings.goal = "পড়াশোনা".into();
        s.save("config.json", &c).unwrap();
        assert_eq!(s.load::<Config>("config.json").unwrap(), c);
        fs::write(s.path("config.json"), b"{nope").unwrap();
        assert_eq!(s.load::<Config>("config.json").unwrap(), Config::default());
        assert!(s.path("config.json.corrupt").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
