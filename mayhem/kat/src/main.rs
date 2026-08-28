// KAT (known-answer test) probe for the behavioral oracle (SPEC §6.3).
//
// Renders three fixed templates through the UNMODIFIED minijinja crate and prints
// their exact rendered output, one known-answer per line:
//
//   line 1: arithmetic + operator precedence  ->  "7"        (1 + 3*2)
//   line 2: the `upper` builtin filter          ->  "MINIJINJA"
//   line 3: the `range` global + a for-loop     ->  "012"
//
// mayhem/test.sh asserts each line against these exact values. This binary is a
// normal, dynamically-linked Rust executable living under /mayhem, so the gate's
// sabotage shim (LD_PRELOAD constructor that _exit(0)s non-system binaries) neuters
// it — producing empty output and thus a FAILING oracle. That is what proves the
// oracle is behavioral and not merely an exit-code check.
use minijinja::{context, Environment};

fn render(source: &str, ctx: minijinja::Value) -> String {
    let mut env = Environment::new();
    env.add_template("kat", source)
        .expect("KAT template failed to compile");
    env.get_template("kat")
        .expect("KAT template missing")
        .render(ctx)
        .expect("KAT template failed to render")
}

fn main() {
    // 1. arithmetic with operator precedence: 1 + 3 * 2 == 7
    print!("{}\n", render("{{ a + b * 2 }}", context! { a => 1, b => 3 }));
    // 2. the `upper` builtin filter
    print!("{}\n", render("{{ name|upper }}", context! { name => "minijinja" }));
    // 3. the `range` global function driving a for-loop
    print!(
        "{}\n",
        render(
            "{% for i in range(3) %}{{ i }}{% endfor %}",
            context! {},
        )
    );
}
