use std::fs;

use reqwest::get;
use scraper::{Html, Selector};

use crate::constants::{ARCHIVE_BASE, LIST_URL, SHOWS_URL};
use crate::util::sanitize_song_name;

/// Lists all shows available on archive.org
///
/// Shows are listed in chronological order starting from the first recorded
/// soundboard in 1965 to the last soundboard recorded at Soldier Field in 1995.
pub async fn list_shows() -> Result<Vec<String>, reqwest::Error> {
  let title_selector = Selector::parse(r#"div[class="ttl"]"#).unwrap();

  let mut all_boards: Vec<String> = vec![];

  for i in 0..174 {
    let uri = format!("{}{}", SHOWS_URL, i);
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

    for show in &soundboards {
      println!("{}", show);
    }

    all_boards.extend(soundboards);
  }

  Ok(all_boards)
}

/// Attempts to download all songs for the specified show name in ogg vorbis format
///
/// Individual songs will be written to a directory matching the specified show name.
///
/// ## Args:
///
/// `showname` - name as printed by the `list_shows` operation
pub async fn get_show(showname: &String) -> Result<(), reqwest::Error> {
  println!("Attempting to fetch show: {}", showname);

  let title_selector =
    Selector::parse(r#"a[data-event-click-tracking="GenericNonCollection|ItemTile"]"#).unwrap();

  let list_uri = format!("{}{}", LIST_URL, showname);
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

  if urls.len() > 0 {
    let song_selector = Selector::parse(r#"div[itemprop="track"]"#).unwrap();
    let name_selector = Selector::parse(r#"meta[itemprop="name"]"#).unwrap();
    let link_selector = Selector::parse(r#"link[itemprop="associatedMedia"]"#).unwrap();
    let detail_uri = format!("{}{}", ARCHIVE_BASE, &urls[0]);
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
      .filter(|(name, url)| url.ends_with(".ogg") && name != "nothing")
      .collect();
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
          let path = format!("{}/{} - {}.ogg", showname, number, trimmed_name);
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
  } else {
    println!("No shows found with name: {}", showname);
  }

  Ok(())
}

/// Attempts to download soundboards for all shows available on the archive
///
/// Each show will be saved to a directory matching the show name as printed by
/// the `list_shows` operation.
pub async fn get_all_shows() -> Result<(), reqwest::Error> {
  let all_shows = list_shows().await?;
  for show in all_shows {
    match get_show(&show).await {
      Ok(_) => println!("Successfully downloaded show \"{}\"", show),
      Err(e) => println!("Failed to download show \"{}\": {}", show, e),
    };
  }
  Ok(())
}

/// Attempts to download all soundboards recorded between the specified start and end shows
/// 
/// This operation is inclusive, so the specified start and end shows will also be downloaded.
/// If only a single show should be downloaded, use the `get_show` operation instead.
/// 
/// ## Args:
/// 
/// 1. `start` - starting show name for the slice
/// 2. `end` - ending show name for the slice
pub async fn get_slice_shows(start: &String, end: &String) -> Result<(), reqwest::Error> {
  println!(
    "Attempting to download shows from \"{}\" to \"{}\"",
    start, end
  );
  let all_shows = list_shows().await?;

  let mut inside_slice = false;
  for show in all_shows {
    if &show == start {
      inside_slice = true;
    }
    if inside_slice {
      match get_show(&show).await {
        Ok(_) => println!("Successfully downloaded show \"{}\"", show),
        Err(e) => println!("Failed to download show \"{}\": {}", show, e),
      };
    }
    if &show == end {
      inside_slice = false;
    }
  }

  Ok(())
}
