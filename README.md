# Test262 Harness
A rust-lang test harness for the ECMAScript test suite.

## Usage
```toml
# Cargo.toml
[dev.dependencies]
test262-harness = "*"
```

```rust
// lib.rs
#[test]
fn test_js() {
    let test262_path = "test262";
    let harness = Harness::new(test262_path).expect("failed to initialize harness");
    for test in harness {
        println!("running test {} from {}", test.desc.id, test.path);
        if test.desc.negative.is_some() {
            // maybe a parser failure
            // or a runtime failure
        }
    }
}
```

`Test` is the primary way to interact with this crate, it contains
a bulk of the information one would need to test a test262 file.

The `Test` structure includes 3 top-level properties, the path 
to the file being tested, the string contents of that file and
the metadata, contained in the multi-line comment (as YAML) prefix
to all files, about each JavaScript test.