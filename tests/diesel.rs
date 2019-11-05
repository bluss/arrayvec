#![cfg(feature = "diesel")]
#[macro_use]
extern crate diesel;

use arrayvec::ArrayString;
use diesel::{prelude::*, sqlite::SqliteConnection, table};

table! {
    use diesel::sql_types::{Text, Integer};
    texts {
        id -> Integer,
        body -> Text,
    }
}

#[test]
fn test_query() {
    let conn = SqliteConnection::establish(":memory:").unwrap();

    diesel::sql_query("CREATE TABLE texts ( id INTEGER PRIMARY KEY, body TEXT NOT NULL )")
        .execute(&conn)
        .unwrap();

    let body = "Answer to the Ultimate Question of Life, the Universe, and Everything";

    diesel::insert_into(texts::table)
        .values(&(texts::id.eq(42), texts::body.eq(body)))
        .execute(&conn)
        .unwrap();

    let fetched_body = texts::table
        .select(texts::body)
        .load::<ArrayString<[u8; 128]>>(&conn)
        .unwrap()[0];

    assert_eq!(body, fetched_body.as_str());
}
