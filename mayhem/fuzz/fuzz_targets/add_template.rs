// Additive in-process libFuzzer harness for minijinja — parser + compiler.
// Feeds fuzz bytes as UTF-8 template source to Environment::add_template, exercising
// lexing, parsing and compilation. Same behavior as upstream fuzz/add_template.rs.
// No disk I/O; bytes come only from the fuzzer; upstream source is untouched.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &str| {
    minijinja::Environment::new()
        .add_template("fuzz.txt", input)
        .ok();
});
