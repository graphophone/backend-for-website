use axum::extract::Multipart;

use crate::handlers::error::HandlerError;

pub fn asset_key_to_url(key: Option<String>) -> Option<String> {
    key.map(|v| format!("/assets/{v}"))
}

pub struct ImageData {
    pub image_bytes: Vec<u8>,
    pub mime_type: String,
}

pub async fn extract_image_data_from_multipart(multipart: Option<Multipart>) -> Result<Option<ImageData>, HandlerError> {
    let image_data = match multipart {
        Some(mut image_multipart) => {
            let field = image_multipart.next_field()
                .await
                .map_err(|_| HandlerError::BadRequest("Image has to be sent".to_string()))?;            
            let bytes = match field {
                Some(field) => {
                    let mime_type = match field.content_type() {
                        Some(v) => {
                            let split = v.split("/");
                            let mime_type = split.skip(1).next();
                            match mime_type {
                                Some(v) => v.to_string(),
                                None => return Err(HandlerError::BadRequest("Couldn't get mime type".to_string())),
                            }
                        },
                        None => return Err(HandlerError::BadRequest("Couldn't get mime type".to_string())),
                    };

                    let bytes = field.bytes()
                        .await
                        .map_err(|_| {
                            HandlerError::BadRequest("Image has to be sent".to_string())
                        })?
                        .to_vec();
                    
                    (bytes, mime_type)
                },
                None => return Err(HandlerError::BadRequest("Image has to be sent".to_string())),
            };
            Some(bytes)
        },
        None => None,
    };

    match image_data {
        Some((image_bytes, mime_type)) => Ok(Some(ImageData { image_bytes, mime_type })),
        None => Ok(None),
    }
}