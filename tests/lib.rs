use robots::Robots;
use robots::RobotsParser;
use robots::Rule;

#[test]
fn full_robots() {
    //Taken from https://www.robotstxt.org/norobots-rfc.txt Section 4
    let rules = "# /robots.txt for http://www.fict.org/\r
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

    let result = RobotsParser::new(vec![
        Robots::GlobalRule(Rule::Allow("/robots.txt".to_owned())),
        Robots::UserAgent("unhipbot".to_owned(), vec![Rule::Disallow("/".to_owned())]),
        Robots::UserAgent("webcrawler".to_owned(), vec![Rule::Allow("*".to_owned())]),
        Robots::UserAgent("excite".to_owned(), vec![Rule::Allow("*".to_owned())]),
        Robots::UserAgent(
            "*".to_owned(),
            vec![
                Rule::Disallow("/org/plans.html".to_owned()),
                Rule::Allow("/org/".to_owned()),
                Rule::Allow("/serv".to_owned()),
                Rule::Allow("/~mak".to_owned()),
                Rule::Disallow("/".to_owned()),
            ],
        ),
    ]);
    let parsed = RobotsParser::parse(rules);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();
    assert_eq!(parsed, result);
}

#[test]
fn full_path_check() {
    //Taken from https://www.robotstxt.org/norobots-rfc.txt Section 4
    let rules = "# /robots.txt for http://www.fict.org/\r
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

    let parsed = RobotsParser::parse(rules);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();

    assert!(!parsed.can_fetch("*", "http://www.fict.org/"));
    assert!(!parsed.can_fetch("*", "http://www.fict.org/index.html"));
    assert!(parsed.can_fetch("*", "http://www.fict.org/robots.txt"));
    assert!(parsed.can_fetch("*", "http://www.fict.org/server.html"));
    assert!(parsed.can_fetch("*", "http://www.fict.org/services/fast.html"));
    assert!(parsed.can_fetch("*", "http://www.fict.org/services/slow.html"));
    assert!(!parsed.can_fetch("*", "http://www.fict.org/orgo.gif"));
    assert!(parsed.can_fetch("*", "http://www.fict.org/org/about.html"));
    assert!(!parsed.can_fetch("*", "http://www.fict.org/org/plans.html"));
    assert!(!parsed.can_fetch("*", "http://www.fict.org/%7Ejim/jim.html"));
    assert!(parsed.can_fetch("*", "http://www.fict.org/%7Emak/mak.html"));

    assert!(!parsed.can_fetch("unhipbot", "http://www.fict.org/"));
    assert!(!parsed.can_fetch("unhipbot", "http://www.fict.org/index.html"));
    assert!(parsed.can_fetch("unhipbot", "http://www.fict.org/robots.txt"));
    assert!(!parsed.can_fetch("unhipbot", "http://www.fict.org/server.html"));
    assert!(!parsed.can_fetch("unhipbot", "http://www.fict.org/services/fast.html"));
    assert!(!parsed.can_fetch("unhipbot", "http://www.fict.org/services/slow.html"));
    assert!(!parsed.can_fetch("unhipbot", "http://www.fict.org/orgo.gif"));
    assert!(!parsed.can_fetch("unhipbot", "http://www.fict.org/org/about.html"));
    assert!(!parsed.can_fetch("unhipbot", "http://www.fict.org/org/plans.html"));
    assert!(!parsed.can_fetch("unhipbot", "http://www.fict.org/%7Ejim/jim.html"));
    assert!(!parsed.can_fetch("unhipbot", "http://www.fict.org/%7Emak/mak.html"));

    assert!(parsed.can_fetch("webcrawler", "http://www.fict.org/"));
    assert!(parsed.can_fetch("webcrawler", "http://www.fict.org/index.html"));
    assert!(parsed.can_fetch("webcrawler", "http://www.fict.org/robots.txt"));
    assert!(parsed.can_fetch("webcrawler", "http://www.fict.org/server.html"));
    assert!(parsed.can_fetch("webcrawler", "http://www.fict.org/services/fast.html"));
    assert!(parsed.can_fetch("webcrawler", "http://www.fict.org/services/slow.html"));
    assert!(parsed.can_fetch("webcrawler", "http://www.fict.org/orgo.gif"));
    assert!(parsed.can_fetch("webcrawler", "http://www.fict.org/org/about.html"));
    assert!(parsed.can_fetch("webcrawler", "http://www.fict.org/org/plans.html"));
    assert!(parsed.can_fetch("webcrawler", "http://www.fict.org/%7Ejim/jim.html"));
    assert!(parsed.can_fetch("webcrawler", "http://www.fict.org/%7Emak/mak.html"));

    assert!(parsed.can_fetch("excite", "http://www.fict.org/"));
    assert!(parsed.can_fetch("excite", "http://www.fict.org/index.html"));
    assert!(parsed.can_fetch("excite", "http://www.fict.org/robots.txt"));
    assert!(parsed.can_fetch("excite", "http://www.fict.org/server.html"));
    assert!(parsed.can_fetch("excite", "http://www.fict.org/services/fast.html"));
    assert!(parsed.can_fetch("excite", "http://www.fict.org/services/slow.html"));
    assert!(parsed.can_fetch("excite", "http://www.fict.org/orgo.gif"));
    assert!(parsed.can_fetch("excite", "http://www.fict.org/org/about.html"));
    assert!(parsed.can_fetch("excite", "http://www.fict.org/org/plans.html"));
    assert!(parsed.can_fetch("excite", "http://www.fict.org/%7Ejim/jim.html"));
    assert!(parsed.can_fetch("excite", "http://www.fict.org/%7Emak/mak.html"));
}
