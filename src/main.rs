use rraw::{Client, auth::AnonymousAuthenticator, utils::options::FeedOption};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    // https://www.reddit.com/user/KyleKKent/submitted/

    let client =
        Client::login(AnonymousAuthenticator::new(), "RRAW Test (by u/KingTuxWH)").await?;

    println!("Logged in");

    let user = client.user("KyleKKent").await?;

    println!("Got user");

    let mut submissions = vec![];
    let mut prev = None;

    const COUNT: u32 = 25;
    const MAX: usize = usize::MAX;

    loop {
        let subs = user.submissions(Some(FeedOption {
            count: Some(COUNT),
            limit: Some(COUNT),

            after: prev,
            before: None,
            period: None
        })).await?.data;

        prev = subs.after;
        let subs = subs.children;
        let len = subs.len();

        submissions.extend(subs);

        if len < (COUNT as usize) || submissions.len() > MAX {
            break;
        }
    }
    println!("Got Posts");

    for post in submissions {
        println!("{}", post.data.selftext);
    }


    Ok(())
}
