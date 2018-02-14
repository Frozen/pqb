//#[macro_use]
//extern crate log;

//extern crate env_logger;

extern crate postgres;

use postgres::Connection;
use postgres::types::ToSql;
use postgres::rows::Rows;

pub mod prelude {
    pub use {convert_table_name, insert, update, DbModel, SelectQuery};
}

pub trait DbModel {
    fn table(&self) -> String;
    fn fields() -> Vec<&'static str>;
    fn instance_fields(&self) -> Vec<&'static str>;
    fn as_map(&self) -> std::collections::HashMap<&'static str, &ToSql>;
}

pub trait TableName {
    fn table(&self) -> (String, Option<String>);
}

impl<'a> TableName for &'a str {
    fn table(&self) -> (String, Option<String>) {
        (self.to_string(), None)
    }
}

impl<'a> TableName for (&'a str, &'a str) {
    fn table(&self) -> (String, Option<String>) {
        (self.0.to_string(), self.1.to_string().into())
    }
}

//pub trait ToSqlArray<'a> {
//    fn to_array(&self) -> &'a [&'a ToSql];
//}

//impl

#[derive(Default)]
pub struct SelectQuery<'a> {
    _fields: Vec<&'a str>,
    _where: Vec<(&'static str, usize)>,
    _to_sql: Vec<&'a ToSql>,
    _from: String,
    _from_alias: Option<String>,
    _limit: Option<usize>,
    _offset: Option<usize>,
    _join: Vec<(&'a TableName, &'a str)>,
    _group_by: Vec<&'a str>,
    _having: Option<&'a str>,
}

impl<'a> SelectQuery<'a> {
    pub fn new() -> SelectQuery<'a> {
        SelectQuery::default()
    }

    pub fn from_model<M>(m: &M) -> SelectQuery<'a>
    where
        M: DbModel,
    {
        SelectQuery {
            _from: m.table(),
            _fields: m.instance_fields(),
            ..Default::default()
        }
    }

    pub fn from_model_with_alias<M>(m: &M, alias: &'a str) -> SelectQuery<'a>
    where
        M: DbModel,
    {
        SelectQuery {
            _from: m.table(),
            _fields: m.instance_fields(),
            _from_alias: Some(alias.to_string()),
            ..Default::default()
        }
    }

    pub fn alias(&mut self, alias: &'a str) -> &mut SelectQuery<'a> {
        self._from_alias = Some(alias.to_string());
        self
    }

    pub fn filter(&mut self, cond: &'static str, value: &'a ToSql) -> &mut SelectQuery<'a> {
        self._where.push((cond, 1));
        self._to_sql.push(value);
        self
    }

    pub fn filtern(&mut self, cond: &'static str, value: &'a [&'a ToSql]) -> &mut SelectQuery<'a> {
        self._where.push((cond, value.len()));
        self._to_sql.extend(value);
        self
    }

    pub fn limit(&mut self, limit: usize) -> &mut SelectQuery<'a> {
        self._limit = Some(limit);
        self
    }

    pub fn offset(&mut self, offset: usize) -> &mut SelectQuery<'a> {
        self._offset = Some(offset);
        self
    }

    pub fn query(&mut self, conn: &Connection) -> postgres::Result<Rows> {
        let (s, q) = self.build();
        conn.query(&s, &q)
    }

    pub fn join(&mut self, a: &'a TableName, on: &'a str) -> &mut SelectQuery<'a> {
        self._join.push((a, on));
        self
    }

    pub fn fields(&mut self, fields: &'a [&'a str]) -> &mut SelectQuery<'a> {
        self._fields = fields.to_owned();
        self
    }

    pub fn group_by(&mut self, group_by: &'a str) -> &mut SelectQuery<'a> {
        self._group_by.push(group_by);
        self
    }

    pub fn having(&mut self, having: &'a str) -> &mut SelectQuery<'a> {
        self._having = Some(having);
        self
    }

    pub fn build(&self) -> (String, Vec<&'a ToSql>) {
        let mut s = String::new();
        s.push_str("SELECT");

        if self._fields.len() > 0 {
            s.push_str(" ");
            s.push_str(&self._fields.join(", "));
        } else {
            s.push_str(" *")
        }

        s.push_str(" FROM \"");
        s.push_str(&self._from);
        s.push_str("\"");

        if let Some(ref a) = self._from_alias {
            s.push_str(" ");
            s.push_str(a);
        }

        for &(join, on) in &self._join {
            s.push_str(" JOIN ");
            s.push_str("\"");
            s.push_str(&join.table().0);
            s.push_str("\"");

            if let Some(ref alias) = join.table().1 {
                s.push_str(" ");
                s.push_str(alias);
            }

            s.push_str(" ON ");
            s.push_str(on);
        }

        if self._where.len() > 0 {
            s.push_str(" WHERE ");
        }

        let mut index = 1;
        for &(cond, size) in self._where.iter() {
            if index > 1 {
                s.push_str(" AND ");
            }

            let mut n = vec![];
            for _ in 0..size {
                n.push(format!("${}", index));
                index += 1;
            }

            s.push_str(&cond.replace("?", &format!("{}", &n.join(", "))));
        }

        if self._group_by.len() > 0 {
            s.push_str(" GROUP BY ");
            s.push_str(&self._group_by.join(", "));
        }

        if let Some(having) = self._having {
            s.push_str(" HAVING ");
            s.push_str(having);
        }

        if let Some(v) = self._limit {
            s.push_str(" LIMIT ");
            s.push_str(&v.to_string())
        }

        if let Some(v) = self._offset {
            s.push_str(" OFFSET ");
            s.push_str(&v.to_string());
        }

        return (s, self._to_sql.clone());
    }
}

impl<'a> std::fmt::Debug for SelectQuery<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.build().0)
    }
}

pub trait AsMap {
    fn as_map(&self) -> std::collections::HashMap<&'static str, &ToSql>;
}

pub fn insert<T>(conn: &Connection, db_model: &T, skip: &[&str]) -> postgres::Result<Rows>
where
    T: DbModel,
{
    let mut s = String::from("INSERT INTO ");
    let mut v = String::from("");
    let mut q = vec![];

    s.push_str("\"");
    s.push_str(&db_model.table());
    s.push_str("\"");
    s.push_str("(");

    let mut previous = false;
    let mut index = 1;
    for (field, value) in &db_model.as_map() {
        if skip.contains(field) {
            continue;
        }

        if previous {
            s.push_str(", ");
            v.push_str(", ");
        }

        s.push_str("\"");
        s.push_str(field);
        s.push_str("\"");

        v.push_str(&format!("${}", index));
        index += 1;

        q.push(*value);

        previous = true;
    }
    s.push_str(") VALUES (");
    s.push_str(&v);

    s.push_str(")");

    if db_model.instance_fields().len() > 0 {
        s.push_str(" RETURNING ");
        s.push_str("\"");
        s.push_str(&db_model.instance_fields().join("\", \""));
        s.push_str("\"");
    }

    //    println!("{:?}", s);

    conn.query(&s, &q)
}

pub fn update<T>(conn: &Connection, db_model: &T, skip: &[&str]) -> postgres::Result<Rows>
where
    T: DbModel,
{
    let mut s = String::from("UPDATE ");
    let mut v = String::from("");
    let mut q = vec![];

    s.push_str("\"");
    s.push_str(&db_model.table());
    s.push_str("\"");
    s.push_str(" SET ");

    let mut previous = false;
    let mut index = 1;
    for (field, value) in &db_model.as_map() {
        if skip.contains(field) {
            continue;
        }

        if previous {
            s.push_str(", ");
            v.push_str(", ");
        }

        s.push_str("\"");
        s.push_str(field);
        s.push_str("\" = ");

        s.push_str(&format!("${}", index));
        index += 1;

        q.push(*value);

        previous = true;
    }
    s.push_str(" WHERE ");
    s.push_str(&v);

    s.push_str(")");

    if db_model.instance_fields().len() > 0 {
        s.push_str(" RETURNING ");
        s.push_str("\"");
        s.push_str(&db_model.instance_fields().join("\", \""));
        s.push_str("\"");
    }

    conn.query(&s, &q)
}

pub fn convert_table_name(name: &str) -> String {
    let mut s = String::new();
    for x in name.chars().enumerate() {
        match x {
            (0, e) => s.push_str(&e.to_lowercase().collect::<String>()),
            (_, e) => {
                if e >= 'A' && e <= 'Z' {
                    s.push('_');
                    s.push_str(&e.to_lowercase().collect::<String>())
                } else {
                    s.push(e)
                }
            }
        }
    }
    s
}

#[cfg(test)]
mod test {

    #[test]
    fn test_convert_table_name() {
        assert_eq!("my_table_name", super::convert_table_name("MyTableName"));
    }

}
