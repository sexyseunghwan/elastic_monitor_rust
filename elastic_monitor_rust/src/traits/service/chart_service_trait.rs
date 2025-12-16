use crate::common::*;

#[async_trait]
pub trait ChartService {
    async fn generate_line_chart(
        &self,
        title: &str,
        x_labels: Vec<String>,
        y_data: Vec<i64>,
        output_path: &Path,
        x_label: &str,
        y_label: &str,
    ) -> anyhow::Result<()>;
    async fn convert_images_to_base64_html(
        &self,
        alarm_image_path: PathBuf,
    ) -> anyhow::Result<String>;
}
