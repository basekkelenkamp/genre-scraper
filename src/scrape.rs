use reqwest;
use scraper::{Html, Selector};
use std::error::Error;
use regex::Regex;


const BASE_URL: &str = "https://everynoise.com";

#[derive(Debug)]
pub struct GenrePageData {
    pub artists: Vec<ExtractedArtist>,
    pub similar_genres: Vec<String>,
    pub opposite_genres: Vec<String>,
}

#[derive(Debug)]
pub struct ExtractedArtist {
    pub name: String,
    pub audio_link: String,
}


pub async fn scrape_genres() -> Result<Vec<String>, reqwest::Error> {
    let html = reqwest::get(BASE_URL).await?.text().await?;

    let document = Html::parse_document(&html);
    let selector = Selector::parse(".genre.scanme").unwrap();

    let mut genres: Vec<String> = Vec::new();
    for element in document.select(&selector) {
        let genre_text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if !genre_text.is_empty() {
            genres.push(genre_text);
        }
    }

    let cleaned_genres: Vec<String> = genres
        .into_iter()
        .map(|genre| genre.trim_end_matches(" »").to_string())
        .collect();

    Ok(cleaned_genres)
}

pub async fn scrape_genre_page(genre: &String) -> Result<GenrePageData, Box<dyn Error>> {
    let url = format!("{}/engenremap-{}.html", BASE_URL, 
        Regex::new(r"[^a-zA-Z0-9]").unwrap().replace_all(&genre.replace(" ", ""), "")
    );

    let html: String = reqwest::get(url).await?.text().await?;
    let document = Html::parse_document(&html);

    let selector = Selector::parse(".canvas").unwrap();
    let canvases: Vec<_> = document.select(&selector).collect();

    if canvases.len() == 3 {
        let similar_genres_canvas = &canvases[1];
        let opposite_genres_canvas = &canvases[2];

        let extracted_artists: Vec<ExtractedArtist> = extract_artists_from_canvas(&canvases[0]);
        let extracted_similar_genres: Vec<String> = extract_genres_from_canvas(similar_genres_canvas);
        let extracted_opposite_genres: Vec<String> = extract_genres_from_canvas(opposite_genres_canvas);
        // Return these 3 objects

        Ok(GenrePageData {
            artists: extracted_artists,
            similar_genres: extracted_similar_genres,
            opposite_genres: extracted_opposite_genres,
        })

    } else {
        println!("HTML: {}", html);
        println!("Error: Expected 3 canvases, found {}", canvases.len());
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Expected 3 canvases, found {}", canvases.len()))))
    }
}

fn extract_artists_from_canvas(artist_canvas: &scraper::ElementRef) -> Vec<ExtractedArtist> {
    let divs = Selector::parse(".genre.scanme").unwrap();

    let mut artists: Vec<ExtractedArtist> = Vec::new();

    for div in artist_canvas.select(&divs) {
        if let Some(title) = div.value().attr("title") {
            let artist_name = extract_artist_name_from_title(title);
            if let Some(preview_url) = div.value().attr("preview_url") {         
                artists.push(ExtractedArtist {
                    name: artist_name.to_string(),
                    audio_link: preview_url.to_string(),
                });
            }
        }
    }

    artists
}

fn extract_genres_from_canvas(genre_canvas: &scraper::ElementRef) -> Vec<String> {
    let divs = Selector::parse(".genre").unwrap();

    let mut genres: Vec<String> = Vec::new();
    for element in genre_canvas.select(&divs) {
        let genre_text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if !genre_text.is_empty() {
            genres.push(genre_text);
        }
    }

    let cleaned_genres: Vec<String> = genres
        .into_iter()
        .map(|genre| genre.trim_end_matches(" »").to_string())
        .collect();

    cleaned_genres
}

fn extract_artist_name_from_title(title: &str) -> String {
    title.split("e.g.").last()
         .and_then(|s| s.split('"').next()) // Split by the first double quote
         .map(|s| s.trim()) // Trim whitespace
         .unwrap_or_default()
         .to_string()
}
