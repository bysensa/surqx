use surreal_query_macros::sql;

fn main() {
    let name = "";
    let (query, vars) = sql! {
        CREATE person SET name = &name;
        SELECT * FROM person;
    };
    println!("{}", query);
}
