#![allow(clippy::absurd_extreme_comparisons, clippy::missing_errors_doc)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use crowbook::Book;
use rraw::{auth::AnonymousAuthenticator, utils::options::FeedOption, Client};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_vec_pretty};
use std::path::Path;
use tokio::fs::{read_to_string, write};

pub type AnError = Box<dyn std::error::Error>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Response {
    pub title: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostsOfAUser {
    pub username: String,
    pub rsps: Vec<Response>,
}

pub async fn get_posts_correct_order(
    cache_file: impl AsRef<Path> + Clone,
) -> Result<PostsOfAUser, AnError> {
    match read_to_string(cache_file.clone())
        .await
        .map(|x| from_str::<PostsOfAUser>(&x))
    {
        Ok(Ok(posts)) if !posts.rsps.is_empty() => {
            println!("Using cached");
            return Ok(posts);
        }
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("IO Error in cache: {e:?}");
            }
        }
        Ok(Err(e)) => {
            eprintln!("Serde Error in cache: {e:?}");
        }
        _ => {}
    };

    println!("Fetching fresh");
    let client = Client::login(AnonymousAuthenticator::new(), "RRAW Test (by u/KingTuxWH)").await?;
    println!("\tLogged in");
    let user = client.user("KyleKKent").await?;
    println!("\tGot user");

    let mut submissions = vec![];
    let mut prev = None;

    const COUNT: u32 = 100;
    const MAX: usize = usize::MAX;

    loop {
        let subs = user
            .submissions(Some(FeedOption {
                count: Some(COUNT),
                limit: Some(COUNT),

                after: prev,
                before: None,
                period: None,
            }))
            .await?
            .data;

        prev = subs.after;
        let subs = subs.children;
        let len = subs.len();

        submissions.extend(subs.into_iter().map(|x| Response {
            title: x.data.title,
            text: x.data.selftext,
        }));

        println!("\t Got {}", submissions.len());
        if len < (COUNT as usize) || submissions.len() > MAX {
            break;
        }
    }
    submissions.reverse(); //get into chronological order
    println!("\t Got Posts");

    let res = PostsOfAUser {
        username: user.user.name,
        rsps: submissions,
    };
    write(cache_file, to_vec_pretty(&res)?).await?;

    Ok(res)
}

fn to_crowbook(PostsOfAUser { username, rsps }: PostsOfAUser) -> Result<Book, AnError> {
    let mut book = Book::new();
    book.set_options(&[
        ("author", username.as_str()),
        ("title", "Out of Cruel Space"),
        ("lang", "en"),
    ]);

    for (i, Response { title: _title, text }) in rsps.into_iter().enumerate() {
        book.add_chapter_from_source(crowbook::Number::Specified((i + 1) as i32), text.as_bytes(), true)?;
    }

    Ok(book)
}

fn to_txt (PostsOfAUser { username: _username, rsps }: PostsOfAUser) -> String {
    let mut res = String::new();
    for (i, Response { title: _title, text }) in rsps.into_iter().enumerate() {
        res.push_str(&format!("Chapter {}\n\n", i + 1));
        res.push_str(&text);
        res.push_str("\n\n");
    }
    res
}

#[tokio::main]
async fn main() -> Result<(), AnError> {
    let subs = get_posts_correct_order("./cache.json").await?;
    let mut book = to_crowbook(subs.clone())?;

    let mut out = vec![];
    book.render_format_to("epub", &mut out)?;
    write("./out.epub", out.clone()).await?;
    book.render_format_to("html", &mut out)?;
    write("./out.html", out).await?;



    write("./out.txt", to_txt(subs)).await?;
    Ok(())
}
