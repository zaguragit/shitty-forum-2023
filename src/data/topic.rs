use super::ThreadID;

pub struct Topic {
    pub about: String,
    pub threads: Vec<ThreadID>,
}

impl Default for Topic {
    fn default() -> Self {
        Self {
            about: Default::default(),
            threads: vec![],
        }
    }
}