use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod error;
pub use error::Error;

/// The test harness
pub struct Harness {
    test_paths: Vec<PathBuf>,
    idx: usize,
}

impl Harness {
    /// Provide the root path for the
    /// test directory from the test262 repository
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

/// A single entry in the
/// test suite
pub struct Test {
    /// The full js text including the
    /// license and metadata comments
    pub source: String,
    /// The full file path that this test
    /// can be found
    pub path: PathBuf,
    /// The parsed metadata from the
    /// file
    pub desc: Description,
}

/// The parsed metadata from the
/// file
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq)]
pub struct Description {
    /// One possible id
    pub id: Option<String>,
    /// One possible id
    pub esid: Option<String>,
    /// One possible id
    pub es5id: Option<String>,
    /// One possible id
    pub es6id: Option<String>,
    /// A longer description of
    /// what the test is trying
    /// to evaluate
    pub info: Option<String>,
    /// A short description of
    /// what the test is trying
    /// to evaluate
    pub description: Option<String>,
    /// Will be `Some` if this
    /// test should fail
    pub negative: Option<Negative>,
    /// If this test relies on an
    /// files in the /harness
    /// directory they will
    /// be included here
    #[serde(default)]
    pub includes: Vec<String>,
    /// If this test needs
    /// to be executed in
    /// a specific way
    /// i.e. as a module or
    /// strict mode only
    #[serde(default)]
    pub flags: Vec<Flag>,
    /// Any locales that
    /// should be respected
    #[serde(default)]
    pub locale: Vec<String>,
    /// If this test relies on any
    /// features
    #[serde(default)]
    pub features: Vec<String>,
}

/// If a test is expected to
/// fail, this describes
/// how it should fail
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq)]
pub struct Negative {
    /// When should this test fail
    pub phase: Phase,
    /// The name of the expected
    /// exception
    #[serde(alias = "type")]
    pub kind: Option<String>,
}

/// Phase for negative tests
#[derive(Debug, Deserialize, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    /// During the parsing step
    Parse,
    /// After parsing but before
    /// evaluation
    Early,
    /// During module resolution
    Resolution,
    /// During evaluation
    Runtime,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Flag {
    /// This test should only be run in strict mode
    OnlyStrict,
    /// This test should not be run in strict mode
    NoStrict,
    /// This test should only be run as a module
    Module,
    /// This test should be run as is, in non-strict mode
    Raw,
    /// This test will be asynchronous,
    /// use doneprintHandle.js
    Async,
    /// This test was procedurally generated
    Generated,
    /// the [[CanBlock]] record must be false
    #[serde(alias = "CanBlockIsFalse")]
    CanBlockIsFalse,
    /// the [[CanBlock]] record must be true
    #[serde(alias = "CanBlockIsTrue")]
    CanBlockIsTrue,
    /// This test may pass in more than
    /// one way depending on implementation
    #[serde(alias = "non-deterministic")]
    NonDeterministic,
}
