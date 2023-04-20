use super::ReplyID;

pub struct Thread {
    pub title: String,
    pub replies: Vec<ReplyID>,
}