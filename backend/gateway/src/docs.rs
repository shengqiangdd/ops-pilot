//! Swagger UI and OpenAPI JSON endpoint.
//!
//! GET /api/docs/openapi.json — returns the OpenAPI spec as JSON
//! GET /api/docs/swagger-ui — returns an HTML page with Swagger UI

use axum::response::{Html, IntoResponse};

/// GET /api/docs/openapi.json — serve OpenAPI spec as JSON.
pub async fn openapi_json() -> impl IntoResponse {
    let yaml = include_str!("../../../docs/openapi.yaml");
    match serde_yaml::from_str::<serde_json::Value>(yaml) {
        Ok(json_value) => axum::Json(json_value).into_response(),
        Err(_) => {
            // Fallback: return the raw YAML as plain text
            (
                axum::http::StatusCode::OK,
                [("content-type", "text/plain")],
                yaml.to_string(),
            )
                .into_response()
        }
    }
}

/// GET /api/docs/swagger-ui — serve Swagger UI HTML page.
pub async fn swagger_ui() -> Html<String> {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>OpsPilot API Documentation</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
  <style>
    body { margin: 0; padding: 0; }
    .swagger-ui .topbar { display: none; }
    .swagger-ui .info .title { font-size: 1.5em; }
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    SwaggerUIBundle({
      url: '/api/docs/openapi.json',
      dom_id: '#swagger-ui',
      presets: [
        SwaggerUIBundle.presets.apis,
        SwaggerUIBundle.SwaggerUIStandalonePreset
      ],
      layout: 'BaseLayout',
      deepLinking: true,
      defaultModelsExpandDepth: 1,
      defaultModelExpandDepth: 1,
    });
  </script>
</body>
</html>"#
        .to_string(),
    )
}
