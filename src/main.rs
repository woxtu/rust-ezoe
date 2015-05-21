#![feature(convert, plugin)]
#![plugin(docopt_macros, regex_macros)]

extern crate docopt;
extern crate hyper;
extern crate regex;
extern crate rss;
extern crate rustc_serialize;

use std::io::prelude::*;
use docopt::Docopt;

const BLUE: &'static str = "\x1b[34m";
const CYAN: &'static str = "\x1b[36m";
const RED: &'static str = "\x1b[31m";
const RESET: &'static str = "\x1b[0m";

static HIGHLIGHT: regex::Regex = regex!("不?自由|質問ではない。?");

docopt!(Args, "
Usage:
  ezoe [(--user <user>)] [<question>]
  ezoe --help

Options:
  -u, --user  Assign the user name.
  -h, --help  Show this screen.");

fn ask(user: &str, ask: &str) {
  let url = format!("http://ask.fm/{}", user);
  let mut client = hyper::Client::new();
  let mut response = client.get(&url).send().unwrap();

  let mut resource = String::new();
  let _ = response.read_to_string(&mut resource);

  let re = regex!("authenticity_token=' \\+ encodeURIComponent\\('(.+?)'\\)");
  let token = re.captures(&resource).unwrap().at(1).unwrap();

  let url = format!("http://ask.fm/{}/questions/create", user);
  let query =
    [("authenticity_token", token),
     ("question[question_text]", ask),
     ("question[force_anonymous]", "force_anonymous")]
    .iter()
    .map(|&(k, v)| format!("{}={}", k, v))
    .collect::<Vec<_>>()
    .connect("&");

  client.post(&url).body(&query).send().unwrap();
}

fn display(user: &str) {
  let url = format!("http://ask.fm/feed/profile/{}.rss", user);
  let mut client = hyper::Client::new();
  let mut response = client.get(&url).send().unwrap();

  let mut resource = String::new();
  let _ = response.read_to_string(&mut resource);

  let rss::Rss(channel) = resource.parse::<rss::Rss>().unwrap();
  for item in channel.items.into_iter().rev() {
    let link = item.link.unwrap();
    let title = item.title.unwrap();
    let description =
      HIGHLIGHT.replace_all(&item.description.unwrap(),
                            format!("{}$0{}", RED, RESET).as_str());

    println!("{}{}{}", BLUE, link, RESET);
    println!("  {}{}{}", CYAN, title, RESET);
    println!("  {}", description);
    println!("");
  }
}

fn main() {
  let args = Args::docopt().decode::<Args>().unwrap_or_else(|e| e.exit());
  let user = if args.flag_user { args.arg_user } else { "EzoeRyou".to_string() };

  if !args.arg_question.is_empty() {
    ask(&user, &args.arg_question)
  } else {
    display(&user)
  }
}
