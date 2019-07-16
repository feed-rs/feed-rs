use uuid::Uuid;

pub mod element_source;

pub fn uuid_gen() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod test;
