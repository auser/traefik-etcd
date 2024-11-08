pub fn get_safe_key(key: &str) -> String {
    key.replace(".", "-").replace("/", "-")
}
