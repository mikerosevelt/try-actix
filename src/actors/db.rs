use crate::actix::{Actor, Handler, Message, SyncContext};
use crate::diesel::prelude::*;
use crate::models::{NewArticle, Article};
use crate::schema::articles::dsl::{body, articles, published, title, id};
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection
};
use uuid::Uuid;

pub struct DbActor(pub Pool<ConnectionManager<PgConnection>>); 

#[derive(Message)]
#[rtype(result="QueryResult<Article>")]
pub struct Create {
  pub title: String,
  pub body: String,
}

#[derive(Message)]
#[rtype(result="QueryResult<Article>")]
pub struct Update {
  pub id: Uuid,
  pub title: String,
  pub body: String,
}

#[derive(Message)]
#[rtype(result="QueryResult<Article>")]
pub struct Delete {
  pub id: Uuid,
}

#[derive(Message)]
#[rtype(result="QueryResult<Article>")]
pub struct Publish {
  pub id: Uuid,
}

#[derive(Message)]
#[rtype(result="QueryResult<Vec<Article>>")]
pub struct GetArticles;

impl Actor for DbActor {
  type Context = SyncContext<Self>;
}

impl Handler<Create> for DbActor {
  type Result = QueryResult<Article>;

  fn handle(&mut self, msg: Create, _: &mut Self::Context) -> Self::Result {
      let binding = self.0.get();
      let conn = binding.as_ref().expect("Unable to get connection");
      let new_article = NewArticle {
        id: Uuid::new_v4(),
        title: msg.title,
        body: msg.body,
      };

      diesel::insert_into(articles).values(new_article).get_result::<Article>(conn)
  }
}

impl Handler<Update> for DbActor {
  type Result = QueryResult<Article>;

  fn handle(&mut self, msg: Update, _: &mut Self::Context) -> Self::Result {
      let binding = self.0.get();
      let conn = binding.expect("Unable to get connection");

      diesel::update(articles)
      .filter(id.eq(msg.id))
      .set((title.eq(msg.title), body.eq(msg.body)))
      .get_result::<Article>(&conn)
  }
}

impl Handler<Delete> for DbActor {
  type Result = QueryResult<Article>;

  fn handle(&mut self, msg: Delete, _: &mut Self::Context) -> Self::Result {
      let binding = self.0.get();
      let conn = binding.expect("Unable to get connection");

      diesel::delete(articles).filter(id.eq(msg.id)).get_result::<Article>(&conn)
  }
}

impl Handler<Publish> for DbActor {
  type Result = QueryResult<Article>;

  fn handle(&mut self, msg: Publish, _: &mut Self::Context) -> Self::Result {
      let binding = self.0.get();
      let conn = binding.expect("Unable to get connection");

      diesel::update(articles)
      .filter(id.eq(msg.id))
      .set(published.eq(true))
      .get_result::<Article>(&conn)
  }
}

impl Handler<GetArticles> for DbActor {
  type Result = QueryResult<Vec<Article>>;

  fn handle(&mut self, _msg: GetArticles, _: &mut Self::Context) -> Self::Result {
      let binding = self.0.get();
      let conn = binding.expect("Unable to get connection");

      articles.filter(published.eq(true)).get_results::<Article>(&conn)
  }
}


