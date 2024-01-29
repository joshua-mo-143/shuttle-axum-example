use axum::{routing::get, Router, response::IntoResponse, http::StatusCode, extract::{State, Path}, Json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use sqlx::Executor;
use serde::{Deserialize, Serialize};

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[derive(Clone)]
pub struct AppState {
    db: PgPool
}

impl AppState {
    fn new(db: PgPool) -> Self {
        Self { db }
    }
}  

#[derive(sqlx::FromRow, Serialize)]
pub struct User {
    id: i32,
    name: String,
    age: i32
}

#[derive(Deserialize)]
pub struct UserSubmission {
    name: String,
    age: i32
}

#[derive(Deserialize)]
pub struct UpdateRecord {
    name: Option<String>,
    age: Option<i32>
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] conn_string: String 
    ) -> shuttle_axum::ShuttleAxum {
    let db = PgPoolOptions::new()
            .max_connections(5)
            .min_connections(5)
            .connect(&conn_string)
            .await
            .expect("Couldn't connect to the database :(");


    db.execute(include_str!("../migrations.sql")).await.expect("Couldn't run the migrations :(");

    let state = AppState::new(db);

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/users", get(retrieve_all_records).post(create_record))
        .route("/users/:id", get(retrieve_record_by_id)
               .put(update_record_by_id)
               .delete(delete_record_by_id))
        .with_state(state);

    Ok(router.into())
}

async fn retrieve_all_records(
    State(state): State<AppState>
    ) -> Result<impl IntoResponse, impl IntoResponse> {

    let res = match sqlx::query_as::<_, User>("SELECT * FROM USERS")
        .fetch_all(&state.db)
        .await {
            Ok(res) => res,
            Err(e) => {return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Something went wrong: {e}")));} 
        };

    Ok(Json(res))
} 

async fn retrieve_record_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>
    ) -> Result<impl IntoResponse, impl IntoResponse> {

    let res = match sqlx::query_as::<_, User>("SELECT * FROM USERS WHERE ID = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await {
            Ok(res) => res,
            Err(e) => {return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Something went wrong: {e}")));} 
        };

    Ok(Json(res))
} 

async fn create_record(
    State(state): State<AppState>,
    Json(json): Json<UserSubmission>
    ) -> StatusCode {

    sqlx::query("INSERT INTO USERS (name, age) VALUES ($1, $2)")
        .bind(json.name)
        .bind(json.age)
        .execute(&state.db)
        .await.unwrap();

    StatusCode::CREATED 
} 

async fn update_record_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(json): Json<UpdateRecord>
    ) -> StatusCode {

    sqlx::query("UPDATE USERS 
                SET
                (case when $1 is not null then name = $1 else name = name end),
                (case when $2 is not null then age = $2 else age = age end)
                WHERE id = $3")
        .bind(json.name)
        .bind(json.age)
        .bind(id)
        .execute(&state.db)
        .await.unwrap();

    StatusCode::CREATED 
} 
async fn delete_record_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>
    ) -> StatusCode {

    sqlx::query("DELETE FROM USERS WHERE ID = $1")
        .bind(id)
        .execute(&state.db)
        .await.unwrap();

    StatusCode::OK 
} 
