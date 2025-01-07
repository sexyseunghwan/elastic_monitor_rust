use crate::common::*;

use crate::model::Indicies::*;

pub trait MessageFormatter {
    fn get_telegram_format(&self) -> String;
    fn get_email_format(&self) -> HtmlContents;
}

#[derive(Debug, new)]
pub struct HtmlContents {
    pub html_form_map: HashMap<String, String>,
    pub view_page_dir: String,
}

#[derive(Debug, new)]
pub struct MessageFormatterNode {
    pub cluster_name: String,
    pub host: Vec<String>,
    pub err_subject: String,
    pub err_detail: String,
}

impl MessageFormatter for MessageFormatterNode {
    #[doc = "Telgram 형식으로 변환해주는 함수"]
    fn get_telegram_format(&self) -> String {
        let mut msg_contents: String = String::new();
        msg_contents.push_str(format!("==== Error Alert [{}] ====\n", self.cluster_name).as_str());
        msg_contents.push_str(format!("[cluster name]\n{}\n\n", self.cluster_name).as_str());

        msg_contents.push_str(format!("[err_subject]\n{}\n\n", self.err_subject).as_str());
        msg_contents.push_str(format!("[err_detail]\n{}\n\n", self.err_detail).as_str());

        let host_str = self.host.join("\n");
        msg_contents.push_str(format!("[host]\n{}\n\n", host_str).as_str());

        msg_contents
    }

    #[doc = "Email 형식에 맞게 변환"]
    fn get_email_format(&self) -> HtmlContents {
        let mut html_forms = String::new();

        for host in &self.host {
            let html_form = format!(
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

        let html_contents = HtmlContents::new(html_form_map, "./html/node_info.html".to_string());

        html_contents
    }
}

#[derive(Debug, new)]
pub struct MessageFormatterIndex {
    pub cluster_name: String,
    pub host: Vec<String>,
    pub err_subject: String,
    pub err_index_detail: Vec<Indicies>,
}

impl MessageFormatter for MessageFormatterIndex {
    #[doc = "Html 형식으로 변환해주는 함수 -> Message Too Long 이슈 때문에, Telegram 알람은 간단히 받기로 수정."]
    fn get_telegram_format(&self) -> String {
        let mut err_detailed = String::new();

        for indicies in &self.err_index_detail {
            err_detailed.push_str(&indicies.get_indicies_status());
        }

        let mut msg_contents: String = String::new();
        msg_contents.push_str(format!("==== Error Alert [{}] ====\n", self.cluster_name).as_str());
        msg_contents.push_str(format!("[cluster name]\n{}\n\n", self.cluster_name).as_str());
        msg_contents.push_str(format!("[err_subject]\n{}\n\n", self.err_subject).as_str());
        //msg_contents.push_str(format!("[err_detail]\n{}\n\n", err_detailed).as_str());

        let host_str = self.host.join("\n");
        msg_contents.push_str(format!("[host]\n{}\n\n", host_str).as_str());

        msg_contents
    }

    #[doc = "Email 형식에 맞게 변환"]
    fn get_email_format(&self) -> HtmlContents {
        let mut html_form_map: HashMap<String, String> = HashMap::new();

        let mut cluster_html_forms = String::new();

        for host in &self.host {
            let cluster_html_form = format!(
                "<tr>
                    <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'>{}</td>
                    <td style='border: 1px solid #ddd; padding: 8px; text-align: left; color: red;'>{}</td>
                    <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'>{}</td>
                </tr>",
                self.cluster_name, 
                self.err_subject, 
                host
            );

            cluster_html_forms.push_str(&cluster_html_form);
        }

        html_form_map.insert("cluster_info".to_string(), cluster_html_forms);

        let mut index_html_form = String::new();

        for index in &self.err_index_detail {
            let health_color = index.health.to_lowercase();
            let status_color = if index.status == "OPEN" {
                "green"
            } else {
                "red"
            };

            let inner_html_form = format!(
                "<tr>
                    <td style='border: 1px solid #ddd; padding: 8px; text-align: left;'>{}</td>
                    <td style='border: 1px solid #ddd; padding: 8px; text-align: left; color: {};'>{}</td>
                    <td style='border: 1px solid #ddd; padding: 8px; text-align: left; color: {};'>{}</td>
                </tr>", 
                index.index_name,
                health_color,
                index.health,
                status_color,
                index.status
            );

            index_html_form.push_str(&inner_html_form);
        }

        html_form_map.insert("index_info".to_string(), index_html_form);

        let html_contents = HtmlContents::new(html_form_map, "./html/detail_info.html".to_string());

        html_contents
    }
}
