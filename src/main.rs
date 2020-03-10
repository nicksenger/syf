use std::env;
use std::fs;

use reqwest::get;
use scraper::{Html, Selector};
use tokio::runtime::Runtime;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut runtime = Runtime::new().unwrap();

    if let Some(arg) = args.get(1) {
        match &arg[..] {
            "list" => {
                Runtime::block_on(&mut runtime, get_shows(true));
            }
            "fetch" => {
                if let Some(target) = args.get(2) {
                    if let target = "all" {
                        Runtime::block_on(&mut runtime, get_all_shows());
                    } else {
                        Runtime::block_on(&mut runtime, get_show(target));
                    }
                } else {
                    println!("Please specify a show to fetch or `all`");
                }
            }
            _ => println!(
                "Invalid command. Valid options are `list`, `fetch $showname`, or `fetch all`"
            ),
        };
    } else {
        let message = "You need to specify something to do. \
        Valid options are `list`, `fetch $showname`, or `fetch all`";
        println!("{}", message);
    }
}

async fn get_shows(print: bool) -> Result<Vec<String>, reqwest::Error> {
    let title_selector = Selector::parse(r#"div[class="ttl"]"#).unwrap();

    let mut all_boards: Vec<String> = vec![];

    for i in 0..174 {
        let uri = format!("https://archive.org/details/GratefulDead?and%5B%5D=subject%3A%22Soundboard%22&sort=date&page={}&scroll=1", i);
        let body = get(&uri[..]).await?.text().await?;

        let document = Html::parse_document(&body);
        let mut soundboards: Vec<String> = document
            .select(&title_selector)
            .map(|el| {
                let text = el.text().collect::<Vec<_>>();
                String::from(text.join("").trim())
            })
            .collect();

        soundboards.dedup();

        if print {
            for show in &soundboards {
                println!("{}", show);
            }
        }

        all_boards.extend(soundboards);
    }

    Ok(all_boards)
}

async fn get_show(showname: &String) -> Result<(), reqwest::Error> {
    println!("Attempting to fetch show: {}", showname);

    let title_selector =
        Selector::parse(r#"a[data-event-click-tracking="GenericNonCollection|ItemTile"]"#).unwrap();

    let list_uri = format!("https://archive.org/details/GratefulDead?and%5B%5D={}&sin=&and%5B%5D=subject%3A%22Soundboard%22&sort=-downloads", showname);
    let list_body = get(&list_uri[..]).await?.text().await?;
    let list_document = Html::parse_document(&list_body);
    let urls: Vec<String> = list_document
        .select(&title_selector)
        .map(|el| match el.value().attr("href") {
            Some(link) => String::from(link),
            _ => String::from("nothing"),
        })
        .filter(|s| s != "nothing")
        .collect::<Vec<String>>();

    assert!(urls.len() > 0);

    let song_selector = Selector::parse(r#"div[itemprop="track"]"#).unwrap();
    let name_selector = Selector::parse(r#"meta[itemprop="name"]"#).unwrap();
    let link_selector = Selector::parse(r#"link[itemprop="associatedMedia"]"#).unwrap();

    let detail_uri = format!("{}{}", "https://archive.org/", &urls[0]);
    let detail_body = get(&detail_uri[..]).await?.text().await?;
    let detail_document = Html::parse_document(&detail_body);
    let names_urls: Vec<(String, String)> = detail_document
        .select(&song_selector)
        .map(|el| {
            let mut name = "nothing";
            for child in el.select(&name_selector) {
                if let Some(song_name) = child.value().attr("content") {
                    name = song_name;
                }
            }
            let mut url = "nothing";
            for child in el.select(&link_selector) {
                if let Some(href) = child.value().attr("href") {
                    url = href;
                }
            }
            (String::from(name), String::from(url))
        })
        .filter(|(name, url)| url.ends_with(".ogg"))
        .collect();

    fs::create_dir(showname);

    for (idx, (name, url)) in names_urls.iter().enumerate() {
        let number = if idx < 9 {
            format!("0{}", idx + 1)
        } else {
            format!("{}", idx + 1)
        };
        println!("Downloading {} - {}.ogg", number, name);
        let path = format!("{}/{} - {}.ogg", showname, number, name);
        let file = get(&url[..]).await?.bytes().await;
        match file {
            Ok(b) => {
                fs::write(path, b);
            }
            Err(e) => eprintln!("got an error! {}", e),
        };
    }

    Ok(())
}

async fn get_all_shows() -> Result<(), reqwest::Error> {
    let all_shows = get_shows(true).await?;
    for show in all_shows {
        get_show(&show).await;
    }
    Ok(())
}
