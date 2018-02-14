extern crate postgres;
#[macro_use]
extern crate pqb_codegen;

extern crate pqb;

use pqb::prelude::*;

use postgres::rows::Row;
use postgres::types::ToSql;

#[test]
#[allow(dead_code)]
fn test_model_fields() {
    #[derive(Model, Default)]
    struct MyModel {
        id: i32,
        login: String,
    }

    assert_eq!(
        vec!["id".to_string(), "login".to_string()],
        MyModel::fields()
    );
}

#[test]
fn test_table_name() {
    #[derive(Model, Default)]
    struct MyModel {}

    assert_eq!("SELECT * FROM \"my_model\"", MyModel::select().build().0);
}

#[test]
fn test_struct_without_fields() {
    #[derive(Model, Default)]
    struct NoFieldsStruct();

    let v: Vec<String> = Vec::new();

    assert_eq!(v, NoFieldsStruct::fields());
}

#[test]
fn test_filter() {
    #[derive(Model, Default)]
    struct MyModel {
        id: i32,
        login: String,
    }

    let q = MyModel::select()
        .filter("user_id > ?", &5)
        .filter("bla like ?", &"aaaa")
        .build();

    assert_eq!(
        "SELECT id, login FROM \"my_model\" WHERE user_id > $1 AND bla like $2",
        q.0
    );
}

#[test]
fn test_limit() {
    #[derive(Model, Default)]
    struct MyModel {}

    assert_eq!(
        "SELECT * FROM \"my_model\" LIMIT 1",
        MyModel::select().limit(1).build().0
    );
}

#[test]
fn test_offset() {
    #[derive(Model, Default)]
    struct MyModel {}

    assert_eq!(
        "SELECT * FROM \"my_model\" OFFSET 10",
        MyModel::select().offset(10).build().0
    );
}

#[test]
fn test_complex() {
    #[derive(Model, Default)]
    struct User {}

    let rs = User::alias("u")
        .fields(&["u.id", "p.title"])
        .join(&("post", "p"), "p.user_id = u.id")
        .filtern("u.id in (?)", &[&1, &2, &3])
        .filter("p.id = ?", &10)
        .group_by("u.id")
        .group_by("p.id")
        .having("count(u.id) > 10")
        .limit(10)
        .offset(10)
        .build();

    assert_eq!(
        "SELECT u.id, p.title FROM \"user\" u \
         JOIN \"post\" p ON p.user_id = u.id \
         WHERE u.id in ($1, $2, $3) \
         AND p.id = $4 \
         GROUP BY u.id, p.id \
         HAVING count(u.id) > 10 \
         LIMIT 10 \
         OFFSET 10",
        rs.0
    );

    assert_eq!("[1, 2, 3, 10]", format!("{:?}", rs.1))
}
