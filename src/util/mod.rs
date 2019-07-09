use uuid::Uuid;

pub fn uuid_gen() -> String {
    Uuid::new_v4().to_string()
}
