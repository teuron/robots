use criterion::{criterion_group, criterion_main, Criterion};
use robots::RobotsParser;

fn bench_small_robots(c: &mut Criterion) {
    //Taken from https://www.robotstxt.org/norobots-rfc.txt Section 4
    let input = "# /robots.txt for http://www.fict.org/\r
# comments to webmaster@fict.org\r
\r
User-agent: unhipbot\r
Disallow: /\r
User-agent: webcrawler\r
User-agent: excite\r
Disallow: \r
\r
User-agent: *\r
Disallow: /org/plans.html\r
Allow: /org/\r
Allow: /serv\r
Allow: /~mak\r
Disallow: /";
    c.bench_function("Example Robots.txt", move |b| {
        b.iter(|| RobotsParser::parse(input))
    });
}

criterion_group!(benches, bench_small_robots);

criterion_main!(benches);
