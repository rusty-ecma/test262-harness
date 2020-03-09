use test262_harness::{Flag, Harness, Phase};

#[test]
fn check_all_from_repo() {
    for t in Harness::new("test262").unwrap() {
        t.unwrap();
    }
}

#[test]
fn check_readme_code() {
    for test in Harness::new("test262").unwrap() {
        let test = test.unwrap();
        println!("running test from {:?}", test.path);
        if let Some(id) = &test.desc.id {
            println!("id: {}", id);
        }
        if let Some(id) = &test.desc.esid {
            println!("esid: {}", id);
        }
        if let Some(id) = &test.desc.es5id {
            println!("es5id: {}", id);
        }
        if let Some(id) = &test.desc.es6id {
            println!("es6id: {}", id);
        }
        if let Some(neg) = &test.desc.negative {
            print!("expecting test to fail ");
            if let Some(except) = &neg.kind {
                print!("with {}", except);
            }
            match &neg.phase {
                Phase::Parse => println!("during parsing"),
                Phase::Early => println!("after parsing but before evaluation"),
                Phase::Resolution => println!("while resolving es6 modules"),
                Phase::Runtime => println!("during evaluation"),
            }
        }
        if let Some(info) = &test.desc.info {
            println!("info: {}", info);
        }
        if let Some(desc) = &test.desc.description {
            println!("desc: {}", desc);
        }
        for name in &test.desc.includes {
            println!("import {} from the /harness directory", name);
        }
        for flag in &test.desc.flags {
            match flag {
                Flag::OnlyStrict => println!("This test should only run in strict mode"),
                Flag::NoStrict => println!("This test should not run in strict mode"),
                Flag::Module => println!("This test should be run as a module only"),
                Flag::Raw => {
                    println!("This test's content should not be altered and run as not-strict only")
                }
                Flag::Async => println!("This test needs to be executed asynchronously"),
                Flag::Generated => println!("This test was not written by hand"),
                Flag::CanBlockIsFalse => println!("When executing [[CanBlock]] must be false"),
                Flag::CanBlockIsTrue => println!("[[CanBlock]] must be true"),
                Flag::NonDeterministic => println!("This test can pass in more than one way"),
            }
        }
        for feat in &test.desc.features {
            println!("This test is gated by the feature {}", feat);
        }
    }
}
