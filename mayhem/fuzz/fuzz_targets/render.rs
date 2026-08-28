// Additive in-process libFuzzer harness for minijinja — the full render pipeline.
//
// An Arbitrary-derived (root source, includes, serde Value) tuple is loaded into an
// Environment and rendered. Mirrors upstream fuzz/fuzz_targets/render.rs, but adapted
// to the current minijinja API: `Template::render` takes `V: Into<Value>`, and a serde
// `Serialize` value is converted by wrapping it in `minijinja::value::Serde` (upstream's
// stale `tmpl.render(&value)` no longer compiles against minijinja 3.0.0-alpha.0).
//
// A conservative fuel bound protects against pathological templates burning the run.
// No disk I/O; bytes come only from the fuzzer; upstream source is untouched.
#![no_main]
use std::collections::BTreeMap;

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use serde::Serialize;

#[derive(Debug, Serialize, Arbitrary)]
enum Value {
    None,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(BTreeMap<String, Value>),
}

fuzz_target!(|data: (&str, Vec<(&str, &str)>, Value)| {
    let (root, includes, value) = data;

    let mut env = minijinja::Environment::new();

    // A conservative default fuel so a badly-fuzzed template can't spend the whole run.
    env.set_fuel(Some(50000));

    if env.add_template("fuzz", root).is_err() {
        return;
    }

    for (name, data) in includes {
        let _ = env.add_template(name, data);
    }

    let tmpl = env.get_template("fuzz").unwrap();
    // Convert the serde `Serialize` value into a minijinja Value via the `Serde` wrapper.
    tmpl.render(minijinja::value::Serde(&value)).ok();
});
