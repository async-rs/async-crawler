#![feature(async_await)]

mod html_parser;

use surf;
use url::Url;
use async_std::task;
use html_parser::extract_links;

type CrawlResult =  Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;
type BoxFuture = std::pin::Pin<Box<dyn std::future::Future<Output = CrawlResult> + Send>>;

fn box_crawl(pages: Vec<Url>, current: u8, max_depth: u8) -> BoxFuture {
    Box::pin(crawl(pages, current, max_depth))
}

async fn crawl(pages: Vec<Url>, current: u8, max_depth: u8) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    println!("Depth - Current: {}, Max: {}", current, max_depth);

    if current > max_depth {
        println!("Reached max depth");
        return Ok(());
    }

    let mut tasks = vec![];

    println!("crawling: {:?}", pages);

    for url in pages {
        let task = task::spawn(async move {
            println!("getting: {}", url);

            let mut res = surf::get(&url).await?;
            let body = res.body_string().await?;

            let links = extract_links(&url, body);

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
        crawl(vec![Url::parse("https://www.rust-lang.org/").unwrap()], 1, 2).await
    })
}