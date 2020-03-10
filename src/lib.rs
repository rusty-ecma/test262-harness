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

    fn create_test_from_file(p: &PathBuf) -> Result<Test, Error> {
        let contents = std::fs::read_to_string(p)?;
        Self::create_test(contents, p.clone())
    }
    
    pub(crate) fn create_test(contents: String, path: PathBuf) -> Result<Test, Error> {
        let (yaml_start, yaml_end) = Self::find_yaml(&contents, &path)?;
        let yaml = contents[yaml_start..yaml_end].replace("\r", "\n");
        let license = Self::find_license(&contents)?;
        let desc = serde_yaml::from_str(&yaml)?;
        Ok(Test {
            desc,
            license,
            path,
            source: contents,
        })
    }

    fn find_license(content: &str) -> Result<Option<(usize, usize)>, Error> {
        let pattern = regex::Regex::new(r"(?ix:
        // Copyright( \([C]\))? (\w+) .+\. {1,2}All rights reserved\.[\r\n]{1,2}
            (
                // This code is governed by the( BSD)? license found in the LICENSE file\.
                |
                // See LICENSE for details.
                |
                // Use of this source code is governed by a BSD-style license that can be[\r\n]{1,2}
                // found in the LICENSE file\.
                |
                // See LICENSE or https://github\.com/tc39/test262/blob/master/LICENSE
            )([ \t]*[\r\n]{1,2})*)")?;
        Ok(if let Some(matches) = pattern.find(content) {
            let start = matches.start();
            let end = matches.end();
            Some((start, end))
        } else {
            None
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
            Some(Self::create_test_from_file(p))
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
    license: Option<(usize, usize)>,
}

impl Test {
    pub fn license(&self) -> Option<&str> {
        let (open, end) = self.license?;
        Some(&self.source[open..end])
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_test() {
        let yaml = "
// License!!!
/*---
id: dummytest
esid: dummyesid
es5id: dummyes5id
es6id: dummyes6id
info: |
  a sentence about the stuff
description: a handy description
negative:
  phase: parse
  type: Exception
flags: [noStrict]
locale: [en-us]
features: [BestFeature]
---*/
let x = 'javascript';";
        let result = Harness::create_test(yaml.to_string(), PathBuf::new()).unwrap();
        let Test{desc, ..} = result;
        assert_eq!(desc.id, Some("dummytest".to_string()));
        assert_eq!(desc.esid, Some("dummyesid".to_string()));
        assert_eq!(desc.es5id, Some("dummyes5id".to_string()));
        assert_eq!(desc.es6id, Some("dummyes6id".to_string()));
        assert_eq!(desc.info, Some("a sentence about the stuff\n".to_string()));
        assert_eq!(desc.description, Some("a handy description".to_string()));
        assert_eq!(&desc.flags, &[Flag::NoStrict]);
        assert_eq!(&desc.locale, &["en-us".to_string()]);
        assert_eq!(&desc.features, &["BestFeature".to_string()]);
        let negative = desc.negative.unwrap();
        assert_eq!(negative.phase, Phase::Parse);
        assert_eq!(negative.kind, Some("Exception".to_string()));
    }

    #[test]
    fn can_not_get_license() {
        let yaml = "
// License!!!
/*---
id: dummytest
---*/
let x = 'javascript';
";
        let result = Harness::create_test(yaml.to_string(), PathBuf::new()).unwrap();
        assert_eq!(result.license(), None);
    }

    #[test]
    fn can_get_license() {
        let yaml = "// Copyright (c) 2020 rusty-ecma.  All rights reserved.
// This code is governed by the MIT license found in the LICENSE file.

/*---
id: dummytest
---*/
let x = 'javascript';
";
        let result = Harness::create_test(yaml.to_string(), PathBuf::new()).unwrap();
        assert_eq!(result.license(), Some("// Copyright (c) 2020 rusty-ecma.  All rights reserved.
// This code is governed by the MIT license found in the LICENSE file.
"));
    }
}
