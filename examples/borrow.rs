
fn bla(text: String) -> String {
    format!("bla {text}")
}
fn bla_ref(text: &str) -> String {
    format!("bla {text}")
}


fn main(){

    let text = String::from("Hello");

    let text = bla(text.clone());
    let text_ref = text.as_str();
    bla_ref(text_ref);
    bla_ref(text_ref);
    bla_ref(text_ref);
    bla_ref(text_ref);

}