use test262_harness::Harness;

#[test]
fn check_all_from_repo() {
    for t in Harness::new("test262").unwrap() {
        t.unwrap();
    }
}
