use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use serde_json::json;
use futures::StreamExt;

#[derive(Deserialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    suffix: Option<String>,
    images: Option<Vec<String>>,
    format: Option<String>,
    options: Option<serde_json::Value>,
    system: Option<String>,
    template: Option<String>,
    context: Option<String>,
    stream: Option<bool>,
    raw: Option<bool>,
    keep_alive: Option<String>,
}

// Endpoint para generar una respuesta en base a un prompt y modelo específico
#[post("/api/generate")]
async fn generate_response(req: web::Json<GenerateRequest>) -> impl Responder {
    let client = Client::new();
    let url = "http://localhost:11434/api/generate";

    // Construir el cuerpo de la solicitud JSON con los parámetros avanzados
    let body = json!({
        "model": req.model,
        "prompt": req.prompt,
        "suffix": req.suffix,
        "images": req.images,
        "format": req.format,
        "options": req.options,
        "system": req.system,
        "template": req.template,
        "context": req.context,
        "stream": req.stream.unwrap_or(true),
        "raw": req.raw,
        "keep_alive": req.keep_alive.unwrap_or_else(|| "5m".to_string()),
    });

    // Enviar solicitud POST a Ollama para generar la respuesta
    match client.post(url).json(&body).send().await {
        Ok(response) => {
            if req.stream.unwrap_or(true) {
                let mut stream = response.bytes_stream();

                // Procesar el stream y enviar cada fragmento al cliente en tiempo real
                let response_stream = async_stream::stream! {
                    while let Some(chunk) = stream.next().await {
                        match chunk {
                            Ok(bytes) => {
                                yield Ok::<_, actix_web::Error>(web::Bytes::from(bytes));
                            }
                            Err(_) => {
                                yield Err(actix_web::error::ErrorInternalServerError("Error reading stream"));
                            }
                        }
                    }
                };

                // Configurar la respuesta de streaming
                return HttpResponse::Ok()
                    .content_type("application/json")
                    .streaming(response_stream);
            }

            // Enviar la respuesta completa si `stream` es `false`
            match response.json::<serde_json::Value>().await {
                Ok(json) => HttpResponse::Ok().json(json),
                Err(_) => HttpResponse::InternalServerError().body("Error parsing Ollama response"),
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to connect to Ollama"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(generate_response) // Registra el endpoint para generación de respuestas
    })
        .bind("127.0.0.1:8081")?
        .run()
        .await
}
