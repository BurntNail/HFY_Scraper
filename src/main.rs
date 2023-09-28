use std::path::Path;
use rraw::{auth::AnonymousAuthenticator, utils::options::FeedOption, Client};
use serde::{Serialize, Deserialize};
use serde_json::{from_str, to_vec_pretty};
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
    pub rsps: Vec<Response>
}

pub async fn get_posts_correct_order (cache_file: impl AsRef<Path> + Clone) -> Result<PostsOfAUser, AnError> {
    match read_to_string(cache_file.clone()).await.map(|x| from_str::<PostsOfAUser>(&x) ) {
        Ok(Ok(posts)) if !posts.rsps.is_empty() => {
            println!("Using cached");
            return Ok(posts);
        },
        Err(e) => if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!("IO Error in cache: {e:?}");
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
 
         submissions.extend(subs.into_iter().map(|x| {
            Response {
                title: x.data.title,
                text: x.data.selftext
            }
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
        rsps: submissions
    };
     write(cache_file, to_vec_pretty(&res)?).await?;

     Ok(
        res
     )
 
}

#[tokio::main]
async fn main() -> Result<(), AnError> {
    let subs = get_posts_correct_order("./cache.json").await?;
    println!("Got back {} posts", subs.rsps.len());

    Ok(())
}
