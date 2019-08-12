//! This code is 4 years old and handed down generations.
//! There' probably a better way to do this and maybe even write
//! an iterator over all links, maybe a kind soul will help here.
//! 
//! Upside: it works!

use html5ever::tokenizer::{Token, TagToken, TokenSink, Tokenizer, TokenizerOpts, TokenSinkResult, BufferQueue, TagKind};
use html5ever::tokenizer::Tag;
use url::{Url, ParseError};

use std::borrow::Borrow;

#[derive(Default, Debug)]
struct LinkFinder {
    links: Vec<String>
}

impl TokenSink for &mut LinkFinder {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<Self::Handle> {
        match token {
            TagToken(ref tag @ Tag{kind: TagKind::StartTag, ..}) => {
                if tag.name.as_ref() == "a" {
                    for attr in tag.attrs.iter() {
                        if attr.name.local.as_ref() == "href" {
                            let attr_str: &[u8] = attr.value.borrow();
                            self.links.push(String::from_utf8_lossy(attr_str).into_owned());
                        }
                    }
                }
            },
            _ => {  }
        }
        TokenSinkResult::Continue
    }
}

pub fn extract_links(url: &Url, page: String) -> Vec<Url> {
    let mut domain = url.clone();
    domain.set_path("");
    domain.set_query(None);

    let mut link_finder = LinkFinder::default();
    let mut tokenizer = Tokenizer::new(&mut link_finder, TokenizerOpts::default());
    let mut queue = BufferQueue::new();
    queue.push_back(page.into());
    let _ = tokenizer.feed(&mut queue);
    link_finder.links.iter().map(|link| {
        match Url::parse(link) {
            Err(ParseError::RelativeUrlWithoutBase) => {
                domain.join(link).unwrap()               
            }
            Err(_) => { panic!("Weird link found: {}", link)}
            Ok(url) => url
        }
    }).collect()
}