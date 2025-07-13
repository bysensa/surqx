# SurQX

A Rust library that provides a convenient macro for writing SurrealDB queries with variable interpolation and type-safe parameter binding.

## Features

- **Macro-based query writing**: Write SurrealDB queries using the `sql!` macro with embedded Rust variables
- **Type-safe variable binding**: Automatically serialize Rust variables into SurrealDB-compatible values
- **Variable interpolation**: Use `&variable` syntax to safely interpolate variables into queries
- **Transaction support**: Full support for SurrealDB transactions and complex queries
- **Compile-time validation** [in future]: Catch errors at compile time rather than runtime

P.S: A huge thank you to @m-ou-se for the work done on the inline-python library, which inspired the implementation of this solution.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
surqx = "0.1.0"
surrealdb = "2.3.7"
tokio = { version = "1", features = ["full"] }
```

## Usage

### Basic Example

```rust
use surqx::sql;
use surrealdb::engine::any::connect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = connect("mem://").await?;
    db.use_ns("test").use_db("test").await?;

    let name = "John";
    let age = 30;

    let (query, vars) = sql! {
        CREATE person SET name = &name, age = &age;
        SELECT * FROM person WHERE name = &name;
    };

    let result = db.query(query).bind(vars).await?;
    let result = result.check()?;

    println!("{:?}", result);
    Ok(())
}
```

### Transaction Example

```rust
use surqx::sql;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Person {
    pub name: String,
    pub age: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = connect("mem://").await?;
    db.use_ns("test").use_db("test").await?;

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

    let result = db.query(query).bind(vars).await?;
    let result = result.check()?;

    println!("{:?}", result);
    Ok(())
}
```

### Advanced Query Example

```rust
use surqx::sql;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = connect("mem://").await?;
    db.use_ns("demo").use_db("demo").await?;

    let (query, vars) = sql! {
        CREATE person:aristotle, article:on_sleep_and_sleeplessness;
        RELATE person:aristotle->wrote->article:on_sleep_and_sleeplessness;

        LET $time = time::now();
        RELATE person:author->wrote->article:demo
            CONTENT {
                time: {
                    written: $time
                }
            };

        SELECT * FROM person WHERE ->knows->person->(knows WHERE influencer = true) TIMEOUT 5s;
        SELECT ->purchased->product<-purchased<-person->purchased->product FROM person:tobie PARALLEL;

        SELECT
            name,
            ->(wrote WHERE written_at = "Athens")->book.{ name, id } AS books_written_in_athens
        FROM person;
    };

    let result = db.query(query).bind(vars).await?;
    Ok(())
}
```

## How It Works

The `sql!` macro processes your SurrealDB queries at compile time and:

1. **Extracts variables**: Identifies `&variable` references in your query
2. **Generates unique parameter names**: Creates unique parameter names to avoid collisions
3. **Serializes values**: Converts Rust values to SurrealDB-compatible types using serde
4. **Returns tuple**: Provides both the processed query string and a `Vars` struct for binding

### Variable Interpolation

Use the `&` prefix to interpolate Rust variables into your queries:

```rust
let user_id = "john_doe";
let min_age = 18;

let (query, vars) = sql! {
    SELECT * FROM users
    WHERE id = &user_id AND age >= &min_age;
};
```

This is equivalent to:

```rust
let (query, vars) = sql! {
    SELECT * FROM users
    WHERE id = $user_id_abc123 AND age >= $min_age_def456;
};
// Where vars contains the serialized values for user_id and min_age
```

## Architecture

This library consists of two main components:

- **surqx-macros**: Procedural macro crate that handles query processing and variable extraction
- **surqx**: Main library that provides the `Vars` type and re-exports the macro

The macro uses a custom tokenizer to:

- Parse SurrealDB query syntax
- Extract variable references
- Generate unique parameter names using nanoid
- Preserve query formatting and structure

## Error Handling

The library provides compile-time error checking for:

- Invalid variable references
- Serialization errors
- Query syntax issues

Runtime errors are handled through the `Vars` type, which collects serialization errors and reports them when the query is executed.

## Requirements

- Rust 1.88.0 or later
- SurrealDB 2.3.7 or later

## License

This project is licensed under the Apache 2.0 License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
