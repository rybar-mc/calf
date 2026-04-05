use calf::ApiDoc;
use std::fs;
use utoipa::OpenApi;

const SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>SwaggerUI</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui.css" />
</head>
<body>
<div id="swagger-ui"></div>
<script src="https://unpkg.com/swagger-ui-dist@5.11.0/swagger-ui-bundle.js" crossorigin></script>
<script>
  window.onload = () => {
    window.ui = SwaggerUIBundle({
      url: '/openapi.json',
      dom_id: '#swagger-ui',
    });
  };
</script>
</body>
</html>"#;

fn main() {
    let openapi_json = serde_json::to_string(&ApiDoc::openapi()).unwrap();

    fs::create_dir_all("public/docs").unwrap();
    fs::write("public/openapi.json", openapi_json).unwrap();
    fs::write("public/docs/index.html", SWAGGER_UI_HTML).unwrap();
}
