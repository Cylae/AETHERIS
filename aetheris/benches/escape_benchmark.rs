use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn manual_escape_html(s: &str) -> String {
    let mut output = String::with_capacity(s.len() + 10);
    for c in s.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(c),
        }
    }
    output
}

fn benchmark_escape(c: &mut Criterion) {
    let input_mixed = "<script>alert('XSS')</script> & \"more\"";
    let input_clean = "This is a clean string with no escaping needed.";

    let mut group = c.benchmark_group("escape_html");

    // Case 1: Mixed input (Escaping required)
    group.bench_function("manual_mixed", |b| {
        b.iter(|| {
            manual_escape_html(black_box(input_mixed))
        })
    });

    group.bench_function("html_escape_mixed_owned", |b| {
        b.iter(|| {
            html_escape::encode_safe(black_box(input_mixed)).into_owned()
        })
    });

    // Case 2: Clean input (No escaping required)
    group.bench_function("manual_clean", |b| {
        b.iter(|| {
            manual_escape_html(black_box(input_clean))
        })
    });

    group.bench_function("html_escape_clean_owned", |b| {
        b.iter(|| {
            html_escape::encode_safe(black_box(input_clean)).into_owned()
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_escape);
criterion_main!(benches);
