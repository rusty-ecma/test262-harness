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