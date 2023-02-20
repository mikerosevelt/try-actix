extern crate actix;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod models;
mod db_utils;
mod schema;
mod actors;

use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path, self},
    App, HttpResponse, HttpServer, Responder, HttpRequest,
};

use actix::{SyncArbiter};
use actors::db::{Create, Update, DbActor, Delete, Publish, GetArticles};
use db_utils::{get_pool, run_migrations};
use models::{AppState, ArticleData};
use dotenvy::dotenv;
use std::env;
use uuid::Uuid;

#[post("/articles")]
async fn create_article(article: Json<ArticleData>, state: Data<AppState>) -> impl Responder {
    let db = state.as_ref().db.clone();
    let article = article.into_inner();

    match db.send(Create { title: article.title, body: article.body }).await {
        Ok(Ok(article)) => HttpResponse::Ok().json(article),
        _ => HttpResponse::InternalServerError().json("Something went wrong")
    }
}

#[patch("/articles/{id}")]
async fn update_article(path: Path<(Uuid,)>, article: Json<ArticleData>, state: Data<AppState>) -> impl Responder {
    let db = state.as_ref().db.clone();
    let article = article.into_inner();
    let id = path.into_inner().0;

    match db.send(Update { id, title: article.title, body: article.body }).await {
        Ok(Ok(article)) => HttpResponse::Ok().json(article),
        Ok(Err(_)) => HttpResponse::NotFound().json("Article not found"),
        _ => HttpResponse::InternalServerError().json("Something went wrong")
    }
}

#[delete("/articles/{id}")]
async fn delete_article(path: Path<(Uuid,)>, state: Data<AppState>) -> impl Responder {
    let db = state.as_ref().db.clone();
    let id = path.into_inner().0;

    match db.send(Delete { id }).await {
        Ok(Ok(article)) => HttpResponse::Ok().json(article),
        Ok(Err(_)) => HttpResponse::NotFound().json("Article not found"),
        _ => HttpResponse::InternalServerError().json("Something went wrong")
    }
}

#[post("/articles/{id}/publish")]
async fn publish_article(path: Path<(Uuid,)>, state: Data<AppState>) -> impl Responder {
    let db = state.as_ref().db.clone();
    let id = path.into_inner().0;

    match db.send(Publish { id }).await {
        Ok(Ok(article)) => HttpResponse::Ok().json(article),
        Ok(Err(_)) => HttpResponse::NotFound().json("Article not found"),
        _ => HttpResponse::InternalServerError().json("Something went wrong")
    }
}

#[get("/articles")]
async fn get_articles(state: Data<AppState>) -> impl Responder {
    let db = state.as_ref().db.clone();

    match db.send(GetArticles).await {
        Ok(Ok(articles)) => HttpResponse::Ok().json(articles),
        _ => HttpResponse::InternalServerError().json("Something went wrong")
    }
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let db_url = env::var("DATABASE_URL").expect("Error retrieving the database url");
    run_migrations(&db_url);
    let pool = get_pool(&db_url);
    let addr = SyncArbiter::start(5, move || DbActor(pool.clone()));

    HttpServer::new(move || {
        App::new()
        .route("/", web::get().to(greet))
        // .route("/{name}", web::get().to(greet))
        .service(create_article)
        .service(update_article)
        .service(delete_article)
        .service(publish_article)
        .service(get_articles)
        .app_data(Data::new(AppState {
            db: addr.clone()
        }))
    })
    .bind(("127.0.0.1", 5000))?
    .run()
    .await

}
