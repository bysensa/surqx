pub use surqx_macros::sql;
pub use types::Vars;

mod types;

mod surqx {
    pub use super::sql;
    pub use super::types::Vars;
}

#[cfg(test)]
mod tests {
    use super::surqx;
    use serde::{Deserialize, Serialize};
    use surqx_macros::sql;
    use surrealdb::Surreal;
    use surrealdb::engine::any::{Any, connect};

    async fn db() -> Surreal<Any> {
        let db = connect("mem://").await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_query() {
        let db = db().await;

        let name = "John";
        let age = 18;
        let (query, vars) = sql! {
            CREATE person SET name = &name;
            SELECT * FROM person;
        };
        let query = db.query(query).bind(vars);
        let res = query.await.unwrap();
        let res = res.check().unwrap();
        dbg!(&res);
    }
    #[tokio::test]
    async fn test_specific_query() {
        let db = db().await;

        let name = "John";
        let age = 18;
        let (query, vars) = sql! {
            SELECT id, search::highlight("<b>", "</b>", 1) AS title
            FROM book WHERE title @1@ "rust web";
        };
        let query = db.query(query).bind(vars);
        let res = query.await.unwrap();
        let res = res.check().unwrap();
        dbg!(&res);
    }

    #[tokio::test]
    async fn test_transaction_query() {
        #[derive(Serialize, Deserialize)]
        struct Person {
            pub name: String,
            pub age: usize,
        }

        let db = db().await;

        let persons = vec![
            Person {
                name: "Jane".to_string(),
                age: 23,
            },
            Person {
                name: "John".to_string(),
                age: 32,
            },
        ];

        let (query, vars) = sql! {
            BEGIN TRANSACTION;
            FOR $person IN &persons {
                CREATE type::thing("person", $person.name) CONTENT {
                    name: $person.name,
                    age: $person.age,
                };
            };
            COMMIT TRANSACTION;
            SELECT * FROM person;
        };
        let query = db.query(query).bind(vars);
        let res = query.await.unwrap();
        let res = res.check().unwrap();
        dbg!(&res);
    }
}
