use surqx::{Vars, sql};

fn main() {
    let name = "";
    let (query, vars) = sql! {
        CREATE person SET name = &name;
        SELECT * FROM person;
        RETURN 'r"person:john";
        RETURN 'd"2023-11-28T11:41:20.262Z";
        RETURN 'u"8c54161f-d4fe-4a74-9409-ed1e137040c1";
        RETURN 'f"bucket:/some/key/to/a/file.txt";
    };
    println!("{}", query);
}
