use chrono::{NaiveDateTime, Utc};
use ormx::{Insert, Table};
use sqlx::PgPool;

// To run this example, first run `/scripts/postgres.sh` to start postgres in a docker container and
// write the database URL to `.env`. Then, source `.env` (`. .env`) and run `cargo run`

mod query2;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()?;

    let db = PgPool::connect(&dotenv::var("DATABASE_URL")?).await?;

    log::info!("insert a new row into the database");
    let mut new = InsertUser {
        first_name: "Moritz".to_owned(),
        last_name: "Bischof".to_owned(),
        email: "moritz.bischof1@gmail.com".to_owned(),
        role: Role::User
    }
    .insert(&mut *db.acquire().await?)
    .await?;

    log::info!("update a single field");
    new.set_last_login(&db, Some(Utc::now().naive_utc()))
        .await?;

    log::info!("update all fields at once");
    new.email = "asdf".to_owned();
    new.update(&db).await?;

    log::info!("apply a patch to the user, updating its first and last name");
    new.patch(
        &db,
        UpdateName {
            first_name: "NewFirstName".to_owned(),
            last_name: "NewLastName".to_owned(),
        },
    )
    .await?;

    log::info!("reload the user, in case it has been modified");
    new.reload(&db).await?;

    log::info!("use the improved query macro for searching users");
    let search_result = query2::query_users(&db, Some("NewFirstName"), None).await?;
    println!("{:?}", search_result);

    log::info!("delete the user from the database");
    new.delete(&db).await?;

    Ok(())
}

#[derive(Debug, ormx::Table)]
#[ormx(table = "users", id = user_id, insertable)]
struct User {
    // map this field to the column "id"
    #[ormx(column = "id")]
    #[ormx(get_one = get_by_user_id)]
    user_id: i32,
    first_name: String,
    last_name: String,
    // generate `User::by_email(&str) -> Result<Option<Self>>`
    #[ormx(get_optional(&str))]
    email: String,
    #[ormx(custom_type)]
    role: Role,
    // don't include this field into `InsertUser` since it has a default value
    // generate `User::set_last_login(Option<NaiveDateTime>) -> Result<()>`
    #[ormx(default, set)]
    last_login: Option<NaiveDateTime>,
}

// Patches can be used to update multiple fields at once (in diesel, they're called "ChangeSets").
#[derive(ormx::Patch)]
#[ormx(table_name = "users", table = crate::User, id = "id")]
struct UpdateName {
    first_name: String,
    last_name: String,
}

#[derive(Debug, Copy, Clone, sqlx::Type)]
enum Role {
    User,
    Admin,
}