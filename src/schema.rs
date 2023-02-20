// @generated automatically by Diesel CLI.

diesel::table! {
    articles (id) {
        id -> Uuid,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}
