use std::fs;

use reqwest::get;
use scraper::{Html, Selector};
use serde::Deserialize;
use thirtyfour::prelude::*;

use crate::constants::{ARCHIVE_BASE, LIST_URL, SHOWS_URL};
use crate::util::sanitize_song_name;

/// Lists all shows available on archive.org
///
/// Shows are listed in chronological order starting from the first recorded
/// soundboard in 1965 to the last soundboard recorded at Soldier Field in 1995.
pub async fn list_shows() -> Result<Vec<String>, reqwest::Error> {
    let mut all_names = std::collections::HashSet::new();

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await.unwrap();
    driver.goto(SHOWS_URL).await.expect("go");

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let app_root = driver.find(By::Tag("app-root")).await.expect("app-root");
    let collection_page = app_root
        .get_shadow_root()
        .await
        .expect("shadow")
        .find(By::Tag("collection-page"))
        .await
        .expect("collection-page");
    let collection_browser = collection_page
        .get_shadow_root()
        .await
        .expect("shadow")
        .find(By::Tag("collection-browser"))
        .await
        .expect("collection-browser");
    let infinite_scroller = collection_browser
        .get_shadow_root()
        .await
        .expect("shadow")
        .find(By::Tag("infinite-scroller"))
        .await
        .expect("collection-browser");

    let mut tiles = infinite_scroller
        .get_shadow_root()
        .await
        .expect("shadow")
        .find_all(By::Tag("tile-dispatcher"))
        .await
        .expect("tiles");
    let mut prev = String::new();
    let mut last = tiles.last().expect("tile");
    let mut last_name = last.text().await.expect("moar");

    while last_name != prev {
        for tile in &tiles {
            let name = tile.text().await.expect("text");
            let name = name.split("\n").next().expect("title").to_string();
            let a = tile
                .get_shadow_root()
                .await
                .expect("shadow")
                .find(By::Tag("a"))
                .await
                .expect("a");
            let href = a
                .attr("href")
                .await
                .expect("href")
                .expect("href")
                .replace("/details/", "");

            let mut rev_chars = name.chars().rev();
            let date = (&mut rev_chars)
                .take(10)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>();

            let _ = (&mut rev_chars).take(4).collect::<Vec<_>>();
            let name = rev_chars
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>();

            if all_names.insert(name.clone()) {
                println!("\"{} - {}\" \"{}\"", date, name, href);
            }
        }
        prev = last_name.clone();
        last.scroll_into_view().await.expect("scroll");
        tokio::time::sleep(std::time::Duration::from_millis(4000)).await;

        tiles = infinite_scroller
            .get_shadow_root()
            .await
            .expect("shadow")
            .find_all(By::Tag("tile-dispatcher"))
            .await
            .expect("tiles");

        last = tiles.last().expect("tile");
        last_name = last.text().await.expect("moar");
    }

    driver.quit().await.expect("quit");

    Ok(all_names.into_iter().collect())
}

/// Attempts to download all songs for the specified show name in ogg vorbis format
///
/// Individual songs will be written to a directory matching the specified show name.
///
/// ## Args:
///
/// `showname` - name as printed by the `list_shows` operation
pub async fn get_show(showname: &String, showpath: &String) -> Result<(), reqwest::Error> {
    println!("Attempting to fetch show: {}", showname);

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await.unwrap();
    driver
        .goto(format!("{}/details/{}", ARCHIVE_BASE, showpath))
        .await
        .expect("go");

    let input = driver
        .find(By::ClassName("js-ia-metadata"))
        .await
        .expect("stuff");

    #[derive(Debug, Deserialize)]
    struct Details {
        files: Vec<File>,
    }

    #[derive(Debug, Deserialize)]
    struct File {
        name: String,
        #[serde(default)]
        track: Option<String>,
    }

    let s = input.attr("value").await.expect("attr").expect("attr");
    let json: Details = serde_json::from_str(&s).expect("json");
    let files = json.files.into_iter().filter_map(|f| {
        f.track.as_ref().and_then(|track| {
            f.name
                .ends_with(".mp3")
                .then(|| (track.clone(), f.name.clone()))
        })
    });
    let names_urls = files
        .map(|(track, name)| {
            (
                format!("{}-{}", track, name),
                format!("{}download/{}/{}", ARCHIVE_BASE, showpath, name),
            )
        })
        .collect::<Vec<_>>();

    driver.quit().await.expect("quit");

    match fs::create_dir(showname) {
        Ok(_) => {
            for (idx, (name, url)) in names_urls.iter().enumerate() {
                let sanitized_name = sanitize_song_name(name);
                let trimmed_name = sanitized_name.trim();
                let number = if idx < 9 {
                    format!("0{}", idx + 1)
                } else {
                    format!("{}", idx + 1)
                };
                let path = format!("{}/{}", showname, name);
                match get(&url[..]).await?.bytes().await {
                    Ok(b) => match fs::write(path, b) {
                        Ok(_) => println!(
                            "Successfully downloaded \"{} - {}.ogg\"",
                            number, trimmed_name
                        ),
                        Err(e) => println!(
                            "Failed to download \"{} - {}.ogg\": {}",
                            number, trimmed_name, e
                        ),
                    },
                    Err(e) => eprintln!("got an error! {}", e),
                };
            }
        }
        Err(e) => println!(
            "Failed to create directory for show \"{}\": {}",
            showname, e
        ),
    }

    // let title_selector =
    //     Selector::parse(r#"a[data-event-click-tracking="GenericNonCollection|ItemTile"]"#).unwrap();

    // let list_uri = format!("{}{}", LIST_URL, showname);
    // let list_body = get(&list_uri[..]).await?.text().await?;
    // let list_document = Html::parse_document(&list_body);
    // let urls: Vec<String> = list_document
    //     .select(&title_selector)
    //     .map(|el| match el.value().attr("href") {
    //         Some(link) => String::from(link),
    //         _ => String::from("nothing"),
    //     })
    //     .filter(|s| s != "nothing")
    //     .collect::<Vec<String>>();

    // if urls.len() > 0 {
    //     let song_selector = Selector::parse(r#"div[itemprop="track"]"#).unwrap();
    //     let name_selector = Selector::parse(r#"meta[itemprop="name"]"#).unwrap();
    //     let link_selector = Selector::parse(r#"link[itemprop="associatedMedia"]"#).unwrap();
    //     let detail_uri = format!("{}{}", ARCHIVE_BASE, &urls[0]);
    //     let detail_body = get(&detail_uri[..]).await?.text().await?;
    //     let detail_document = Html::parse_document(&detail_body);
    //     let names_urls: Vec<(String, String)> = detail_document
    //         .select(&song_selector)
    //         .map(|el| {
    //             let mut name = "nothing";
    //             for child in el.select(&name_selector) {
    //                 if let Some(song_name) = child.value().attr("content") {
    //                     name = song_name;
    //                 }
    //             }
    //             let mut url = "nothing";
    //             for child in el.select(&link_selector) {
    //                 if let Some(href) = child.value().attr("href") {
    //                     url = href;
    //                 }
    //             }
    //             (String::from(name), String::from(url))
    //         })
    //         .filter(|(name, url)| url.ends_with(".ogg") && name != "nothing")
    //         .collect();
    //     match fs::create_dir(showname) {
    //         Ok(_) => {
    //             for (idx, (name, url)) in names_urls.iter().enumerate() {
    //                 let sanitized_name = sanitize_song_name(name);
    //                 let trimmed_name = sanitized_name.trim();
    //                 let number = if idx < 9 {
    //                     format!("0{}", idx + 1)
    //                 } else {
    //                     format!("{}", idx + 1)
    //                 };
    //                 let path = format!("{}/{} - {}.ogg", showname, number, trimmed_name);
    //                 match get(&url[..]).await?.bytes().await {
    //                     Ok(b) => match fs::write(path, b) {
    //                         Ok(_) => println!(
    //                             "Successfully downloaded \"{} - {}.ogg\"",
    //                             number, trimmed_name
    //                         ),
    //                         Err(e) => println!(
    //                             "Failed to download \"{} - {}.ogg\": {}",
    //                             number, trimmed_name, e
    //                         ),
    //                     },
    //                     Err(e) => eprintln!("got an error! {}", e),
    //                 };
    //             }
    //         }
    //         Err(e) => println!(
    //             "Failed to create directory for show \"{}\": {}",
    //             showname, e
    //         ),
    //     }
    // } else {
    //     println!("No shows found with name: {}", showname);
    // }

    Ok(())
}

/// Attempts to download soundboards for all shows available on the archive
///
/// Each show will be saved to a directory matching the show name as printed by
/// the `list_shows` operation.
pub async fn get_all_shows() -> Result<(), reqwest::Error> {
    let all_shows = list_shows().await?;
    for show in all_shows {
        match get_show(&show, &Default::default()).await {
            Ok(_) => println!("Successfully downloaded show \"{}\"", show),
            Err(e) => println!("Failed to download show \"{}\": {}", show, e),
        };
    }
    Ok(())
}
