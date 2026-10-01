use axum::{Json, Router, extract::{Query, State}, response::{IntoResponse, Response}, routing::get};

use crate::{clients::categories::{CategoriesClient, categories_grpc}, handlers::{categories::dto::{CategoryData, SearchQuery}, error::HandlerError}};

mod dto;

async fn search_categories(
    query: Query<SearchQuery>,
    State(mut categories_client): State<CategoriesClient>,
) -> Result<Response, HandlerError> {
    let search_token = query.0.search_token;

    let req = categories_grpc::GetCategoriesRequest {
        search_token,
        page_number: 1,
        page_size: 5,
    };

    let res = match categories_client.get_categories(req).await {
        Ok(v) => v.into_inner().categories,
        Err(_) => return Err(HandlerError::NotFound),
    };

    let categories = res.into_iter().map(|c| CategoryData {
        id: c.id,
        name: c.name,
    }).collect::<Vec<CategoryData>>();
    Ok(Json(categories).into_response())
}

pub fn create_categories_router(
    categories_client: CategoriesClient,
) -> Router {
    Router::new()
        .route("/", get(search_categories))
        .with_state(categories_client)
}