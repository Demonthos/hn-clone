#![allow(non_snake_case)]
use dioxus::prelude::*;

fn main() {
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            div { Stories {} }
        }
    })
}

fn Stories(cx: Scope) -> Element {
    let story = use_future(&cx, (), |_| get_stories(10));

    let stories = match story.value() {
        Some(Ok(list)) => rsx! {
            for story in list {
                StoryListing { story: story }
            }
        },
        Some(Err(_)) => return cx.render(rsx! {"An error occured"}),
        None => return cx.render(rsx! {"Loading items"}),
    };

    cx.render(rsx! {
        div {
            stories
        }
    })
}

#[inline_props]
fn StoryListing<'a>(cx: Scope<'a>, story: &'a StoryItem) -> Element {
    let StoryItem {
        title,
        url,
        by,
        score,
        time,
        kids,
        ..
    } = story;

    let url = url.as_deref().unwrap_or_default();
    let hostname = url.trim_start_matches("https://").trim_start_matches("http://").trim_start_matches("www.");
    let score = format!("{score} {}", if *score == 1 { " point" } else { " points" });
    let comments = format!(
        "{} {}",
        kids.len(),
        if kids.len() == 1 {
            " comment"
        } else {
            " comments"
        }
    );
    let time = time.format("%D %l:%M %p");

    cx.render(rsx! {
        div {
            padding: "0.5rem",
            div {
                font_size: "1.5rem",
                a {
                    href: "{url}",
                    "{title}"
                }
                match hostname {
                    "" => rsx!{ "" },
                    name => rsx!{
                        a {
                            color: "gray",
                            href: "https://news.ycombinator.com/from?site={name}",
                            text_decoration: "none",
                            " ({name})"
                        }
                    },
                }
            }
            div {
                display: "flex",
                flex_direction: "row",
                color: "gray",
                div {
                    padding_left: "0.5rem",
                    "{score}"
                }
                div {
                    padding_left: "0.5rem",
                    "by {by}"
                }
                div {
                    padding_left: "0.5rem",
                    "{time}"
                }
                div {
                    padding_left: "0.5rem",
                    "{comments}"
                }
            }
        }
    })
}


// Define the API

use futures::future::join_all;

pub static BASE_API_URL: &str = "https://hacker-news.firebaseio.com/v0/";


pub async fn get_story_preview(id: i64) -> Result<StoryItem, reqwest::Error> {
    let url = format!("{}item/{}.json", BASE_API_URL, id);
    Ok(reqwest::get(&url).await?.json().await?)
}

pub async fn get_stories(count: usize) -> Result<Vec<StoryItem>, reqwest::Error> {
    let url = format!("{}topstories.json", BASE_API_URL);
    let stories_ids = reqwest::get(&url).await?.json::<Vec<i64>>().await?;

    let story_futures = stories_ids[..usize::min(stories_ids.len(), count)]
        .iter()
        .map(|&story_id| get_story_preview(story_id));
    let stories = join_all(story_futures)
        .await
        .into_iter()
        .filter_map(|story| story.ok())
        .collect();
    Ok(stories)
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoryItem {
    pub id: i64,
    pub title: String,
    pub url: Option<String>,
    pub text: Option<String>,
    #[serde(default)]
    pub by: String,
    #[serde(default)]
    pub score: i64,
    #[serde(default)]
    pub descendants: i64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
    #[serde(default)]
    pub kids: Vec<i64>,
    pub r#type: String,
}
