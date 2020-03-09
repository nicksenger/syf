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
                Runtime::block_on(&mut runtime, list_shows());
            }
            "fetch" => {
                if let Some(target) = args.get(2) {
                    Runtime::block_on(&mut runtime, get_show(target));
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

async fn list_shows() -> Result<(), reqwest::Error> {
    let title_selector = Selector::parse(r#"div[class="ttl"]"#).unwrap();

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
        for sb in soundboards {
            println!("{}", sb);
        }
    }

    Ok(())
}

async fn get_show(showname: &String) -> Result<(), reqwest::Error> {
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

    for (name, url) in names_urls {
        println!("Downloading {}.ogg to ./{}", name, showname);
        let path = format!("{}/{}.ogg", showname, name);
        let file = get(&url[..]).await?.bytes().await?;
        fs::write(path, file);
    }

    Ok(())
}
