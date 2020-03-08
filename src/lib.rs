use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod error;
pub use error::Error;

pub struct Harness {
    test_paths: Vec<PathBuf>,
    idx: usize,
}

impl Harness {
    pub fn new<P: AsRef<Path>>(test_root: P) -> Result<Self, Error> {
        let test_paths = WalkDir::new(test_root)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| {
                let entry = match e {
                    Err(e) => return Some(Err(e)),
                    Ok(e) => e,
                };
                let path = entry.path();
                if path.is_dir() {
                    None
                } else {
                    let ext = path.extension()?;
                    if ext == "js" {
                        let file_name = path.file_name()?;
                        let file_name = file_name.to_str()?;
                        if file_name.ends_with("_FIXTURE.js") {
                            None
                        } else {
                            Some(Ok(path.to_path_buf()))
                        }
                    } else {
                        None
                    }
                }
            })
            .collect::<Result<Vec<PathBuf>, walkdir::Error>>()?;
        Ok(Self { test_paths, idx: 0 })
    }

    fn create_test_from(&self, p: &PathBuf) -> Result<Test, Error> {
        let contents = std::fs::read_to_string(p)?;
        let (yaml_start, yaml_end) = Self::find_yaml(&contents, p)?;
        let yaml = contents[yaml_start..yaml_end].replace("\r", "\n");
        let desc = serde_yaml::from_str(&yaml)?;
        Ok(Test {
            desc,
            path: p.clone(),
            source: contents,
        })
    }

    fn find_yaml(content: &str, path: &PathBuf) -> Result<(usize, usize), Error> {
        let start = content
            .find("/*---")
            .ok_or_else(|| Error::DescriptionInvalid(path.clone()))?;
        let end = content
            .find("---*/")
            .ok_or_else(|| Error::DescriptionInvalid(path.clone()))?;
        Ok((start + 5, end))
    }
}

impl Iterator for Harness {
    type Item = Result<Test, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.test_paths.len() {
            None
        } else {
            let p = self.test_paths.get(self.idx)?;
            self.idx += 1;
            Some(self.create_test_from(p))
        }
    }
}

pub struct Test {
    pub source: String,
    pub path: PathBuf,
    pub desc: Description,
}

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq)]
pub struct Description {
    pub id: Option<String>,
    pub esid: Option<String>,
    pub es5id: Option<String>,
    pub es6id: Option<String>,
    pub info: Option<String>,
    pub description: Option<String>,
    pub negative: Option<Negative>,
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub flags: Vec<Flag>,
    #[serde(default)]
    pub locale: Vec<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq)]
pub struct Negative {
    phase: Phase,
    #[serde(alias = "type")]
    kind: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Parse,
    Early,
    Resolution,
    Runtime,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Flag {
    OnlyStrict,
    NoStrict,
    Module,
    Raw,
    Async,
    Generated,
    #[serde(alias = "CanBlockIsFalse")]
    CanBlockIsFalse,
    #[serde(alias = "CanBlockIsTrue")]
    CanBlockIsTrue,
    #[serde(alias = "non-deterministic")]
    NonDeterministic,
}
