# prt-ollama-completion-rust
Documentación del Endpoint /api/generate
Este documento describe el funcionamiento del endpoint POST /api/generate en Actix Web. Este endpoint se conecta a Ollama para generar una respuesta a un prompt proporcionado y puede devolver la respuesta en tiempo real (streaming) o como un único objeto JSON, dependiendo del valor del parámetro stream.

1. Estructura de Datos: GenerateRequest
   rust
   Copy code
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
   GenerateRequest: Estructura que define los parámetros de la solicitud, incluyendo:
   model (obligatorio): Nombre del modelo que generará la respuesta.
   prompt (obligatorio): Texto o pregunta que el modelo debe responder.
   suffix: Texto que sigue a la respuesta generada por el modelo.
   images: Lista opcional de imágenes en base64 para modelos multimodales.
   format: Define el formato de respuesta; actualmente solo json es válido.
   Parámetros avanzados:
   options: Parámetros adicionales para el modelo (por ejemplo, temperature).
   system, template, context, raw: Configuración avanzada para personalizar la generación.
   stream: Controla si la respuesta se envía como un stream en tiempo real (true) o como un solo JSON (false).
   keep_alive: Controla el tiempo de permanencia del modelo en memoria; predeterminado a 5m.
2. Definición del Endpoint generate_response
   rust
   Copy code
   #[post("/api/generate")]
   async fn generate_response(req: web::Json<GenerateRequest>) -> impl Responder {
   #[post("/api/generate")]: Define el endpoint como un POST en /api/generate.
   generate_response: Función que maneja la generación de la respuesta.
   req: web::Json<GenerateRequest>: Actix Web convierte la solicitud JSON en un objeto GenerateRequest, permitiendo acceder a los campos configurados en la solicitud.
3. Crear el Cuerpo de la Solicitud para Ollama
   rust
   Copy code
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
   body: Crea el cuerpo JSON de la solicitud, incluyendo todos los parámetros recibidos en GenerateRequest.
   Valores predeterminados:
   stream predeterminado a true.
   keep_alive predeterminado a 5m.
4. Enviar la Solicitud POST a Ollama y Manejar la Respuesta
   rust
   Copy code
   match client.post(url).json(&body).send().await {
   Ok(response) => { /* Manejo de streaming o respuesta completa */ }
   Err(_) => HttpResponse::InternalServerError().body("Failed to connect to Ollama"),
   }
   client.post(url).json(&body).send().await: Envía la solicitud POST a Ollama.
   Manejo de Errores:
   Si la conexión falla, devuelve un error 500 al cliente.
5. Manejo de Respuesta en Streaming (si stream es true)
   rust
   Copy code
   if req.stream.unwrap_or(true) {
   let mut stream = response.bytes_stream();

   let response_stream = async_stream::stream! {
   while let Some(chunk) = stream.next().await {
   match chunk {
   Ok(bytes) => yield Ok::<_, actix_web::Error>(web::Bytes::from(bytes)),
   Err(_) => yield Err(actix_web::error::ErrorInternalServerError("Error reading stream")),
   }
   }
   };

   return HttpResponse::Ok()
   .content_type("application/json")
   .streaming(response_stream);
   }
   response.bytes_stream(): Obtiene los datos en tiempo real desde Ollama.
   Procesamiento del Stream:
   async_stream::stream!: Procesa y envía cada fragmento al cliente en tiempo real.
   yield Ok: Envía cada fragmento de bytes en formato JSON.
   yield Err(...): Si ocurre un error, envía un mensaje de error.
   HttpResponse::Ok().streaming(response_stream): Configura la respuesta como un stream en tiempo real.
6. Respuesta Completa en JSON (si stream es false)
   rust
   Copy code
   match response.json::<serde_json::Value>().await {
   Ok(json) => HttpResponse::Ok().json(json),
   Err(_) => HttpResponse::InternalServerError().body("Error parsing Ollama response"),
   }
   response.json::<serde_json::Value>().await: Si stream es false, deserializa la respuesta completa como un objeto JSON.
   HttpResponse::Ok().json(json): Envía la respuesta JSON completa al cliente.
7. Configuración del Servidor Actix Web
   rust
   Copy code
   #[actix_web::main]
   async fn main() -> std::io::Result<()> {
   HttpServer::new(|| {
   App::new()
   .service(generate_response) // Registra el endpoint de generación
   })
   .bind("127.0.0.1:8080")?
   .run()
   .await
   }
   #[actix_web::main]: Configura la función main para ejecutarse asincrónicamente en Actix Web.
   HttpServer::new: Crea un servidor HTTP de Actix Web.
   .service(generate_response): Registra el endpoint generate_response para manejar solicitudes POST en /api/generate.
   .bind("127.0.0.1:8080"): Configura el servidor para escuchar en localhost:8080.
   .run().await: Inicia el servidor y espera conexiones entrantes.
8. Ejemplo de Solicitud y Respuesta
   curl -X POST http://localhost:8080/api/generate \
   -H "Content-Type: application/json" \
   -d '{
   "model": "llama3.2",
   "prompt": "Why is the sky blue?",
   "stream": true
   }'
