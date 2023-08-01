use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::Index;
use crate::data::data_types::Json as JsonData;
use crate::data::{DataObjectHandle, Database, ModelHandle};
use crate::server::State;

use super::service_provider::ServiceProviderTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct Json(String);

impl Json {
    pub fn new(data_object_name: String) -> Self {
        Self(data_object_name)
    }
}

async fn data_object(
    database: &Database,
    model: &ModelHandle,
    data_object_name: &str,
) -> Result<JsonData> {
    let data_object = database
        .data_object(data_object_name)
        .await?
        .with_context(|| format!("No data object with name '{data_object_name}'."))?;
    model.data_object(&data_object).await.with_context(|| {
        format!(
            "Failed to get json data object '{data_object_name}' for model '{}'.",
            model.name()
        )
    })
}

async fn page(data_object_name: &str, state: &State, query: &serde_json::Value, model: &ModelHandle, index: Index)-> Result<serde_json::Value> {
    let model_name = model.name();
    let json_object = data_object(state.database(), model, data_object_name).await?;
    let query = query.as_object().context("Query is not an object.")?;
    let json = json_object.page(index).await.with_context(|| 
        format!("Failed to get json data object '{data_object_name}' for {index} of model '{model_name}'.", index = index.error_string())
    )?.value;
    if query.is_empty() {
        Ok(json)
    } else if let Some(json_index) = query.get("get") {
        let mut json = json;
        match json_index {
            serde_json::Value::String(json_index) => {
                if let Some(value) = json.get_mut(json_index) {
                    Some(value)
                } else {
                    let int_index = json_index.parse::<usize>().with_context(|| format!("No field '{json_index}' exists and the index is not an integer."))?;
                    json.get_mut(int_index)
                }
            }
            serde_json::Value::Number(json_index) => json.get_mut(json_index.as_u64().context("Query 'get' field is not a u64.")? as usize),
            _ => bail!("Query 'get' field is not a string or a number."),
        }.with_context(|| format!("Failed to get json value '{json_index}' for {index} of model '{model_name}' and data object '{data_object_name}'.", index = index.error_string()))
        .map(serde_json::Value::take)
    } else {
        bail!("Invalid query for json service. Query must be empty or contain a 'get' field. Query: {query:?}")
    }
}

#[async_trait]
impl ServiceProviderTrait for Json {
    async fn required_data_objects(&self, database: &Database) -> Result<Vec<DataObjectHandle>> {
        let Self(ref data_object_name) = self;
        let data_object = database
            .data_object(data_object_name)
            .await?
            .with_context(|| format!("No data object with name '{data_object_name}'. This should have been checked when the service was created."))?;
        Ok(vec![data_object])
    }

    async fn model_page(
        &self,
        _service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model: &ModelHandle,
    ) -> Result<serde_json::Value> {
        let Self(ref data_object_name) = self;
        page(data_object_name, state, query, model, Index::model()).await
    }

    async fn layer_page(
        &self,
        _service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model: &ModelHandle,
        layer_index: u32,
    ) -> Result<serde_json::Value> {
        let Self(ref data_object_name) = self;
        page(data_object_name, state, query, model, Index::layer(layer_index)).await
    }

    async fn neuron_page(
        &self,
        _service_name: &str,
        state: &State,
        query: &serde_json::Value,
        model: &ModelHandle,
        layer_index: u32,
        neuron_index: u32,
    ) -> Result<serde_json::Value> {
        let Self(ref data_object_name) = self;
        page(data_object_name, state, query, model, Index::neuron(layer_index, neuron_index)).await
    }
}
