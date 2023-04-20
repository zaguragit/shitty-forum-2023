use chrono::Utc;

use crate::data::{ThreadID, ReplyID, ModItem, Moderatable};

use super::{DB, store};

impl DB {
    pub fn move_reply_to_inspection(&mut self, thread_id: &ThreadID, reply_id: &ReplyID) {
        let Some(reply) = self.delete_reply(thread_id, reply_id) else {
            return;
        };
        let id = store::gen_inspection_id();
        let Some(user) = self.users.remove(&reply.user) else {
            return;
        };
        self.inspection.insert(id, ModItem {
            moderated: Utc::now(),
            thing: Moderatable::Reply(user, reply, thread_id.clone()),
        });
    }
}