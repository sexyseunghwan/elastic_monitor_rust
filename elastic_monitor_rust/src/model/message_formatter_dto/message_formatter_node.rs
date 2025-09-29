use crate::common::*;

use crate::model::message_formatter_dto::message_formatter::*;

use crate::env_configuration::env_config::*;

#[derive(Debug, new)]
pub struct MessageFormatterNode {
    pub cluster_name: String,
    pub host: Vec<String>,
    pub err_subject: String,
    pub err_detail: String,
}


impl MessageFormatterNode {

    fn get_chat_api_format(&self) -> String {
        let mut msg_contents: String = String::new();
        msg_contents.push_str(format!("==== Error Alert [{}] ====\n", self.cluster_name).as_str());
        msg_contents.push_str(format!("[cluster name]\n{}\n\n", self.cluster_name).as_str());

        msg_contents.push_str(format!("[err_subject]\n{}\n\n", self.err_subject).as_str());
        msg_contents.push_str(format!("[err_detail]\n{}\n\n", self.err_detail).as_str());

        let host_str: String = self.host.join("\n");
        msg_contents.push_str(format!("[host]\n{}\n\n", host_str).as_str());

        msg_contents
    }

}

impl MessageFormatter for MessageFormatterNode {
    #[doc = "Telgram 형식으로 변환해주는 함수"]
    fn get_telegram_format(&self) -> String {
        self.get_chat_api_format()
    }

    #[doc = "Slack 형식으로 변환해주는 함수"]
    fn get_slack_format(&self) -> String {
        self.get_chat_api_format()
    }

    #[doc = "Email 형식에 맞게 변환"]
    fn get_email_format(&self) -> HtmlContents {
        let mut html_forms: String = String::new();

        for host in &self.host {
            let html_form: String = format!(
                "
                <tr>
                    <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'>{}</td>
                    <td style='border: 1px solid #ddd; padding: 8px; text-align: left; color: red;'>{}</td>
                    <td style='border: 1px solid #ddd; padding: 8px; text-align: left; color: red;'>{}</td>
                    <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'>{}</td>
                </tr>
                ",
                self.cluster_name, 
                self.err_subject, 
                self.err_detail, 
                host
            );

            html_forms.push_str(&html_form);
        }

        let mut html_form_map: HashMap<String, String> = HashMap::new();
        html_form_map.insert("cluster_info".to_string(), html_forms);

        let html_format: &once_lazy<String> = &HTML_TEMPLATE_PATH;
        let html_contents: HtmlContents = HtmlContents::new(html_form_map, html_format.to_string());

        html_contents
    }
}
