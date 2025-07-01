use crate::common::*;

use crate::model::message_formatter_dto::message_formatter::*;

use crate::env_configuration::env_config::*;

#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct UrgentAlarmInfo {
    pub host: String,
    pub metirc_name: String,
    pub metic_value_str: String,
}

#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct MessageFormatterUrgent {
    pub cluster_name: String,
    pub urgent_infos: Vec<UrgentAlarmInfo>,
}

impl MessageFormatter for MessageFormatterUrgent {
    #[doc = "Telgram 형식으로 변환해주는 함수"]
    fn get_telegram_format(&self) -> String {
        let mut msg_contents: String = String::new();
        msg_contents.push_str(format!("==== Error Alert [{}] ====\n", self.cluster_name).as_str());
        msg_contents.push_str(format!("[cluster name]\n{}\n\n", self.cluster_name).as_str());

        msg_contents
                .push_str(format!("[err_subject]\n Emergency Indicator Abnormal \n\n").as_str());
        msg_contents
            .push_str("[err_detail]\n");

        for urgent_info in self.urgent_infos() {
            msg_contents.push_str(
                format!(
                    "{}:{} - {}\n",
                    urgent_info.metirc_name(),
                    urgent_info.metic_value_str(),
                    urgent_info.host()
                )
                .as_str(),
            );
        }
        msg_contents
    }

    #[doc = "Email 형식에 맞게 변환"]
    fn get_email_format(&self) -> HtmlContents {
        let mut html_forms: String = String::new();

        for urgent_info in self.urgent_infos() {
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
                format!("There was a problem with '{}' indicators. ", urgent_info.metirc_name()), 
                format!("{}: {}", urgent_info.metirc_name(), urgent_info.metic_value_str()), 
                urgent_info.host()
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
