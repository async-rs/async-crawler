#![feature(async_await)]

use html5ever::tokenizer::{Token, TagToken, TokenSink, Tokenizer, TokenizerOpts, TokenSinkResult, BufferQueue, TagKind};
use html5ever::tokenizer::Tag;

use std::borrow::Borrow;

use surf;
use async_std::task;
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

fn parse_page(page: String) -> Vec<String> {
    let mut link_finder = LinkFinder::default();
    let mut tokenizer = Tokenizer::new(&mut link_finder, TokenizerOpts::default());
    let mut queue = BufferQueue::new();
    queue.push_back(page.into());
    tokenizer.feed(&mut queue);
    link_finder.links
}

type CrawlResult =  Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;
type BoxFuture = std::pin::Pin<Box<dyn std::future::Future<Output = CrawlResult> + Send>>;

fn box_crawl(pages: Vec<String>, current: u8, max_depth: u8) -> BoxFuture {
    Box::pin(crawl(pages, current, max_depth))
}

async fn crawl(pages: Vec<String>, current: u8, max_depth: u8) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    println!("Current: {}, Max: {}", current, max_depth);
    if current > max_depth {
        println!("Reached max depth");
        return Ok(());
    }

    let mut tasks = vec![];

    println!("crawling: {:?}", pages);

    for mut page in pages {
        if !page.starts_with("http") {
            if page.starts_with("/") {
                page = format!("https://rust-lang.org{}", page);
            } else {
                page = format!("https://rust-lang.org/{}", page);
            }
        }

        let task = task::spawn(async move {
            println!("getting: {}", page);
            let mut res = surf::get(page).await?;
            let body = res.body_string().await?;
            let links = parse_page(body);
            println!("following: {:?}", links);
            box_crawl(links, current + 1, max_depth).await
        });
        tasks.push(task);
    }

    for task in tasks.into_iter() {
        task.await?
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    task::block_on(async {
        crawl(vec!["https://www.rust-lang.org/".into()], 1, 2).await
    })
}