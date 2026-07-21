import React, { useEffect, useRef } from 'react';

/**
 * ApiDocs — in-page Swagger UI for OpsPilot API documentation.
 * Loads Swagger UI from CDN and points it at the backend OpenAPI endpoint.
 */
const ApiDocs: React.FC = () => {
  const swaggerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!swaggerRef.current) return;

    // Load Swagger UI CSS
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = 'https://unpkg.com/swagger-ui-dist@5/swagger-ui.css';
    document.head.appendChild(link);

    // Load Swagger UI bundle
    const script = document.createElement('script');
    script.src = 'https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js';
    script.onload = () => {
      if (swaggerRef.current && (window as any).SwaggerUIBundle) {
        (window as any).SwaggerUIBundle({
          url: '/api/docs/openapi.json',
          dom_id: '#swagger-ui-container',
          presets: [
            (window as any).SwaggerUIBundle.presets.apis,
            (window as any).SwaggerUIBundle.SwaggerUIStandalonePreset,
          ],
          layout: 'BaseLayout',
          deepLinking: true,
          defaultModelsExpandDepth: 1,
          defaultModelExpandDepth: 1,
        });
      }
    };
    document.body.appendChild(script);

    return () => {
      document.head.removeChild(link);
      document.body.removeChild(script);
    };
  }, []);

  return (
    <div className="h-full w-full overflow-auto bg-white">
      <div
        id="swagger-ui-container"
        ref={swaggerRef}
        style={{ minHeight: '100vh' }}
      />
    </div>
  );
};

export default ApiDocs;
