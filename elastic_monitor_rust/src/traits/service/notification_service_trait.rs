use crate::common::*;

use crate::model::message_formatter_dto::message_formatter::*;

#[async_trait]
pub trait NotificationService {
    async fn send_alarm_infos<T: MessageFormatter + Sync + Send>(
        &self,
        msg_fmt: &T,
    ) -> Result<(), anyhow::Error>;
}
