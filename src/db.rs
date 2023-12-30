extern crate rusqlite;
use reqwest::Error;
use rusqlite::{Connection, Result};
use serde_json;

use crate::scrape::GenrePageData;

#[derive(Debug)]
pub struct Genre {
    pub id: i32,
    pub name: String,
    pub similar_genres: Option<String>,
    pub opposite_genres: Option<String>,
}

#[derive(Debug)]
pub struct Artist {
    pub id: i32,
    pub genre_id: i32,
    pub name: String,
    pub audio_link: String,
}


pub fn initialize_db() -> Result<Connection> {
    let conn = Connection::open("genres.db")?;

    // Create or update the 'genres' table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS genres (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            similar_genres TEXT,
            opposite_genres TEXT
        )",
        [],
    )?;

    // Create the 'artists' table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS artists (
            id INTEGER PRIMARY KEY,
            genre_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            audio_link TEXT NOT NULL,
            FOREIGN KEY (genre_id) REFERENCES genres (id)
        )",
        [],
    )?;

    Ok(conn)
}

pub fn insert_genres(conn: &mut Connection, names: &[String]) -> Result<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("INSERT INTO genres (name) VALUES (?1)")?;
        for name in names {
            stmt.execute([name])?;
        }
    }
    tx.commit()?;

    Ok(())
}

pub fn select_genres(conn: &Connection) -> Result<Vec<Genre>> {
    let mut stmt = conn.prepare("SELECT id, name, similar_genres, opposite_genres FROM genres")?;
    let genre_iter = stmt.query_map([], |row| {
        Ok(Genre {
            id: row.get(0)?,
            name: row.get(1)?,
            similar_genres: row.get(2)?,
            opposite_genres: row.get(3)?,
        })
    })?;

    let mut genres = Vec::new();
    for genre in genre_iter {
        genres.push(genre?);
    }

    Ok(genres)
}


pub fn store_genre_page(conn: &mut Connection, genre: &Genre, genre_page_data: &GenrePageData) -> Result<(), Box<dyn std::error::Error>> {
    let tx = conn.transaction()?;

    // Serialize similar_genres and opposite_genres into JSON strings
    let similar_genres_json = serde_json::to_string(&genre_page_data.similar_genres)?;
    let opposite_genres_json = serde_json::to_string(&genre_page_data.opposite_genres)?;

    // Update the genre record with the new data
    tx.execute(
        "UPDATE genres SET similar_genres = ?1, opposite_genres = ?2 WHERE id = ?3",
        &[&similar_genres_json, &opposite_genres_json, &genre.id.to_string()],
    )?;

    // Insert artists into the artists table
    for artist in &genre_page_data.artists {
        tx.execute(
            "INSERT INTO artists (genre_id, name, audio_link) VALUES (?1, ?2, ?3)",
            &[&genre.id.to_string(), &artist.name, &artist.audio_link],
        )?;
    }

    tx.commit()?;

    println!("Stored genre page for {}", genre.name);
    Ok(())
}
