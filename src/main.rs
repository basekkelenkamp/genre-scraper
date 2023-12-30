extern crate rusqlite;
mod db;
mod scrape;

use std::env;
use rusqlite::Result;
use db::{initialize_db, insert_genres, select_genres, store_genre_page};
use scrape::{scrape_genres, scrape_genre_page};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = initialize_db()?;
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "scrape:genres" {
        let scraped_genres = scrape_genres().await?;
        insert_genres(&mut conn, &scraped_genres)?;
        println!("Scraped genres successfully, genres: {:?}", scraped_genres.len());
    }

    if args.len() > 1 && args[1] == "scrape:artists" {
        let mut genres = select_genres(&conn)?;
        if genres.len() == 0 {
            println!("No genres found. Scraping genres first.");

            let scraped_genres = scrape_genres().await?;
            insert_genres(&mut conn, &scraped_genres)?;
            println!("Scraped genres successfully, genres: {:?}", scraped_genres.len());

            genres = select_genres(&conn)?;
            }

        // for genre in &genres {
        //     let scraped_genre_page: scrape::GenrePageData = scrape_genre_page(&genre.name).await?;
        //     store_genre_page(&mut conn, &genre, &scraped_genre_page)?;
        //     println!("Scraped genre page for {}", &genre.name)
        // }

        
        let start_from_index = 934; // Starting from the 935th element
        let filtered_genres = genres.into_iter().skip(start_from_index).collect::<Vec<_>>();

        println!("Starting from index: {}", start_from_index);
        for genre in filtered_genres {
            let scraped_genre_page: scrape::GenrePageData = scrape_genre_page(&genre.name).await?;
            store_genre_page(&mut conn, &genre, &scraped_genre_page)?;
            println!("Scraped genre page for {}", &genre.name);
        }
        
    }

    println!("Success! Exiting...");
    Ok(())
}

