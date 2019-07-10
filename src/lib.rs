//! A robots.txt parser and applicability checker for Rust
//!
//! The Parser is implemented and tested after
//! <http://www.robotstxt.org/norobots-rfc.txt>
//!
//! # Usage
//!
//! Add it to your ``Cargo.toml``:
//!
//! ```toml
//! [dependencies]
//! robots-parser = "0.10"
//! ```
//!
//!
//! # Example
//!
//! ```rust,ignore
//!
//! use robots::RobotsParser;
//! use url::Url;
//!
//! fn main() {
//!     let parsed = RobotsParser::parse_url(Url::new("https://www.google.com/robots.txt"))?;
//!     assert!(parsed.can_fetch("*", "https://www.google.com/search/about"));
//! }
//! ```

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::tag_no_case;
use nom::bytes::complete::take_until;
use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while1;
use nom::combinator::cond;
use nom::combinator::map_opt;
use nom::combinator::opt;
use nom::sequence::tuple;
use nom::IResult;

use url::percent_encoding::percent_decode;
use url::Url;

use std::fs;
use std::path::Path;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RobotsParser {
    rules: Vec<Robots>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Robots {
    UserAgent(String, Vec<Rule>),
    GlobalRule(Rule),
}

impl Robots {
    fn is_applicable(&self, agent: &str, path: &str) -> bool {
        match self {
            Robots::UserAgent(s, _) => {
                let cleaned_user_agent = agent.split('/').nth(0).unwrap_or("");
                if s == "*" || *s == cleaned_user_agent.to_lowercase() {
                    true
                } else {
                    false
                }
            }
            Robots::GlobalRule(rule) => rule.is_applicable(path),
        }
    }

    // Precondition: Applicability has been proven
    fn is_allowed(&self, path: &str) -> bool {
        match self {
            Robots::UserAgent(_, rules) => {
                for rule in rules {
                    if rule.is_applicable(path) {
                        return rule.allowed();
                    }
                }
            }
            Robots::GlobalRule(rule) => return rule.allowed(),
        }
        false
    }
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Rule {
    Allow(String),
    Disallow(String),
    Extension,
}

impl Rule {
    fn is_applicable(&self, path: &str) -> bool {
        let own_path = match self {
            Rule::Allow(s) | Rule::Disallow(s) => s,
            _ => "",
        };

        own_path == "*" || path.starts_with(own_path)
    }

    // Precondition: Applicability has been proven
    fn allowed(&self) -> bool {
        match self {
            Rule::Allow(_) => true,
            _ => false,
        }
    }
}
impl RobotsParser {

    /// Creates a new `RobotsParser` from the given `Robots` Rules
    pub fn new(rules: Vec<Robots>) -> RobotsParser {
        RobotsParser { rules }
    }

    /// Parses a robots.txt input string
    pub fn parse<'a>(input: &'a str) -> Result<RobotsParser, &'static str> {
        let mut rules = vec![];
        let mut input = input;

        //Always add a Allow(/robots.txt) at the start
        rules.push(Robots::GlobalRule(Rule::Allow("/robots.txt".to_owned())));

        loop {
            let rulers = alt((
                RobotsParser::comment_line_parser(),
                map_opt(RobotsParser::crlf_parse(), |_| Some(None::<Robots>)),
                RobotsParser::parse_user_agent(),
                map_opt(RobotsParser::parse_rule(), |rule| {
                    Some(Some(Robots::GlobalRule(rule)))
                }),
            ))(input);
            input = match rulers {
                Ok((input, Some(rule))) => {
                    rules.push(rule);
                    input
                }
                Ok((input, None)) => input,
                Err(_) => {
                    return Err("Could not parse Robots.txt");
                }
            };

            // No more input -> Return
            if input.is_empty() {
                break;
            }
        }

        Ok(RobotsParser { rules: rules })
    }

    /// Parses a robots.txt file from the given path
    pub fn parse_path<P: AsRef<Path>>(path: P) -> Result<RobotsParser, &'static str> {
        let data = fs::read_to_string(path).expect("Unable to read file");
        RobotsParser::parse(&data)
    }

    /// Parses a robots.txt file from the given url
    #[cfg(feature = "web")]
    pub fn parse_url<U: Into<Url>>(url: U) -> Result<RobotsParser, &'static str> {
        let data = reqwest::get(url.into()).expect("Unable to read file from url").text().expect("Unable to rad file from url");
        RobotsParser::parse(&data)
    }

    /// Parses a space
    fn space_parser<'a>() -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
        take_while(|c| c == ' ' || c == '\t')
    }

    // Parses an alphanumeric token or `*`
    fn token_parser<'a>() -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
        take_while1(|c: char| c.is_ascii_alphanumeric() || c == '*')
    }

    /// Parses a comment and does not consume the linebreak
    fn comment_parser<'a>() -> impl Fn(&'a str) -> IResult<&'a str, (&'a str, &'a str)> {
        tuple((tag("#"), take_until("\r\n")))
    }

    /// Parses a line break
    fn crlf_parse<'a>() -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
        tag("\r\n")
    }

    /// Parses a comment line and returns an empty Robots.txt line
    fn comment_line_parser<'a>() -> impl Fn(&'a str) -> IResult<&'a str, Option<Robots>> {
        map_opt(
            tuple((RobotsParser::comment_parser(), RobotsParser::crlf_parse())),
            |_| Some(None),
        )
    }

    fn parse_user_agent<'a>() -> impl Fn(&'a str) -> IResult<&'a str, Option<Robots>> {
        move |input: &'a str| {
            let (input, _) = tag_no_case("user-agent:")(input)?;
            let (input, _) = RobotsParser::space_parser()(input)?;
            let (input, agent) = RobotsParser::token_parser()(input)?;
            // Parses optional comment after path
            let (input, _) = opt(RobotsParser::comment_parser())(input).unwrap_or((input, None));
            let (input, _) = RobotsParser::crlf_parse()(input)?;

            let (input, rules) = RobotsParser::parse_rules()(input)?;

            let rules = if rules.is_empty() {
                //There could be a second User-Agents
                let user_agent = RobotsParser::parse_user_agent()(input);

                let rules = match user_agent {
                    Ok((_, agent)) => match agent.unwrap() {
                        Robots::UserAgent(_, rules) => rules.clone(),
                        _ => panic!("User-Agent only retunrs a User-Agent"),
                    },
                    _ => rules,
                };
                rules
            } else {
                rules
            };
            Ok((input, Some(Robots::UserAgent(agent.to_owned(), rules))))
        }
    }

    /// Parses as many rules it can find
    fn parse_rules<'a>() -> impl Fn(&'a str) -> IResult<&'a str, Vec<Rule>> {
        move |input: &'a str| {
            let mut rules = vec![];
            let mut input = input;
            loop {
                input = match RobotsParser::parse_rule()(input) {
                    Ok((input, rule)) => {
                        rules.push(rule);
                        input
                    }
                    Err(_) => match RobotsParser::comment_line_parser()(input) {
                        Ok((input, _)) => input,
                        Err(_) => return Ok((input, rules)),
                    },
                };
            }
        }
    }

    /// Parses exactly one rule
    fn parse_rule<'a>() -> impl Fn(&'a str) -> IResult<&'a str, Rule> {
        move |input: &'a str| {
            let (input, allowence) = alt((tag("Allow:"), tag("Disallow:")))(input)?;
            let (input, _) = RobotsParser::space_parser()(input)?;
            let (input, path) = RobotsParser::parse_file_path(input)?;

            // Parses optional comment after path
            let (input, _) = opt(RobotsParser::comment_parser())(input).unwrap_or((input, None));

            // CRLF is optional, when the file is empty
            let (input, _) = cond(input.len() != 0, RobotsParser::crlf_parse())(input)?;

            // Empty Disallow means allow all
            if allowence == "Disallow:" && path.is_empty() {
                return Ok((input, Rule::Allow("*".to_owned())));
            }

            match allowence {
                "Allow:" => Ok((input, Rule::Allow(path))),
                "Disallow:" => Ok((input, Rule::Disallow(path))),
                _ => panic!("Rule must either be allowed or disallowed"),
            }
        }
    }

    /// Parses a path as specified
    /// Paths do not include `#` as they indicate a comment
    fn parse_file_path<'a>(input: &'a str) -> IResult<&'a str, String> {
        let (input, path) = take_while(|c: char| !c.is_whitespace() && c != '#')(input)?;
        Ok((input, path.to_owned()))
    }

    /// Decides if a path can be fetched by an agent
    pub fn can_fetch(&self, agent: &str, path: &str) -> bool {
        let url = Url::parse(path);
        match url {
            Ok(url) => {
                let path = percent_decode(url.path().as_bytes()).decode_utf8().unwrap();
                for rule in &*self.rules {
                    if rule.is_applicable(agent, &path) {
                        return rule.is_allowed(&path);
                    }
                }
                false
            }
            Err(_) => return false,
        }
    }
}

#[test]
fn user_agent_different_spellings() {
    assert!(RobotsParser::parse_user_agent()("User-Agent: test\r\n\r\n").is_ok());
    assert!(RobotsParser::parse_user_agent()("user-agent: test\r\n\r\n").is_ok());
    assert!(RobotsParser::parse_user_agent()("USER-AGENT: test\r\n\r\n").is_ok());
}

#[test]
fn user_agent_empty() {
    assert!(RobotsParser::parse_user_agent()("User-Agent:\r\n").is_err());
}

#[test]
fn comment() {
    assert!(RobotsParser::comment_parser()("# testtest\r\n").is_ok());
    assert!(RobotsParser::comment_parser()("testtest\r\n").is_err());
    assert!(RobotsParser::comment_parser()("#testtest").is_err());
    assert!(RobotsParser::comment_line_parser()("# testtest\r\n").is_ok());
    assert!(RobotsParser::comment_line_parser()("testtest\r\n").is_err());
    assert!(RobotsParser::comment_line_parser()("#testtest").is_err());
}

#[test]
fn rule() {
    assert!(RobotsParser::parse_rule()("Allow: /\r\n").is_ok());
    assert!(RobotsParser::parse_rule()("Disallow: /\r\n").is_ok());
    assert!(RobotsParser::parse_rule()("Allow: /#1234 \r\n").is_ok());
    assert!(RobotsParser::parse_rule()("Disallow: /\r\n").is_ok());
    assert!(RobotsParser::parse_rule()("Disallow: \r\n").is_ok());
    assert!(RobotsParser::parse_rule()("Disallow: /org/plans.html\r\n").is_ok());
    assert!(RobotsParser::parse_rule()("Disallow: /org/\r\n").is_ok());
    assert!(RobotsParser::parse_rule()("Allow: /serv\r\n").is_ok());
    assert!(RobotsParser::parse_rule()("Allow: /~mak\r\n").is_ok());
    assert!(RobotsParser::parse_rule()("Allow: /~mak\r\n").is_ok());
}

#[test]
fn rules() {
    let rules = "Disallow: /index.html?\r\nDisallow: /?\r
Allow: /?hl=\r
Disallow: /?hl=*&\r
Allow: /?hl=*&gws_rd=ssl$\r
Disallow: /?hl=*&*&gws_rd=ssl\r
Allow: /?gws_rd=ssl$";
    let result = vec![
        Rule::Disallow("/index.html?".to_owned()),
        Rule::Disallow("/?".to_owned()),
        Rule::Allow("/?hl=".to_owned()),
        Rule::Disallow("/?hl=*&".to_owned()),
        Rule::Allow("/?hl=*&gws_rd=ssl$".to_owned()),
        Rule::Disallow("/?hl=*&*&gws_rd=ssl".to_owned()),
        Rule::Allow("/?gws_rd=ssl$".to_owned()),
    ];
    let parsed = RobotsParser::parse_rules()(rules);
    assert!(parsed.is_ok());
    let (_, parsed) = parsed.unwrap();
    assert_eq!(parsed, result);
}
