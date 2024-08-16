use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MyGreetArgs<'a> {
    pub name: &'a str,
}
