use std::path::Path;

use actix_web::{
    get, http::header::ContentType, rt, web, App, HttpResponse, HttpServer, Responder,
};

use crate::data::NeuroscopePage;

#[get("/api/solu-1l/neuroscope/{layer_index}/{neuron_index}")]
async fn index(indices: web::Path<(u32, u32)>) -> impl Responder {
    let (layer_index, neuron_index) = indices.into_inner();
    let path = Path::new("data")
        .join("solu-1l")
        .join("neuroscope")
        .join(format!("l{layer_index}n{neuron_index}.postcard",));
    match NeuroscopePage::from_file(path) {
        Ok(page) => HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::to_string(&page)
                .expect("Failed to serialize page to JSON. This should always be possible."),
        ),
        Err(error) => HttpResponse::ServiceUnavailable().body(format!("{error}")),
    }
}

pub fn start_server() -> std::io::Result<()> {
    rt::System::new().block_on(
        HttpServer::new(|| App::new().service(index))
            .bind(("127.0.0.1", 8080))?
            .run(),
    )
}
