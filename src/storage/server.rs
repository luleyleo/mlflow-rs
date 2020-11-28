use crate::storage::{
    errors,
    primitive::{Experiment, Run, Metric},
};
use anyhow::{anyhow, Context};
use errors::{CreateExperimentError, GetExperimentError, StorageError};
use nanoserde::{DeJson, SerJson};
use std::convert::TryInto;

use super::BufferedMetric;

pub struct Storage {
    endpoints: Endpoints,
}

struct Endpoints {
    pub experiments_create: String,
    pub experiments_list: String,
    pub experiments_get: String,
    pub runs_create: String,
    pub runs_update: String,
    pub runs_log_param: String,
    pub runs_log_metric: String,
    pub runs_log_batch: String,
}

impl Storage {
    pub fn new(url: &str) -> Self {
        Storage {
            endpoints: Endpoints::new(url),
        }
    }
}

impl Endpoints {
    pub fn new(base: &str) -> Self {
        let api = format!("{}/2.0/mlflow", base);
        Endpoints {
            experiments_create: format!("{}/experiments/create", api),
            experiments_list: format!("{}/experiments/list", api),
            experiments_get: format!("{}/experiments/get-by-name", api),
            runs_create: format!("{}/runs/create", api),
            runs_update: format!("{}/runs/update", api),
            runs_log_param: format!("{}/runs/log-parameter", api),
            runs_log_metric: format!("{}/runs/log-metric", api),
            runs_log_batch: format!("{}/runs/log-batch", api),
        }
    }
}

fn validate_response(response: ureq::Response) -> Result<ureq::Response, StorageError> {
    if response.error() {
        let status = response.status();
        let body = response.into_string()?;
        return Err(anyhow::anyhow!(
            "request failed with status code {}. Body: {}",
            status,
            body
        ));
    }
    Ok(response)
}

impl super::Storage for Storage {
    fn create_experiment(&self, name: &str) -> Result<Experiment, CreateExperimentError> {
        use api::create_experiment::{Request, Response};
        let request = Request::new(name.to_string()).serialize_json();
        let endpoint = &self.endpoints.experiments_create;
        let http_response = ureq::post(endpoint).send_string(&request);
        if http_response.error() {
            let status = http_response.status();
            let body = http_response
                .into_string()
                .context("turning error response into string")?;
            return Err({
                if status == 400 {
                    let response = api::ErrorResponse::deserialize_json(&body)
                        .context("deserializing error body")?;
                    match response.error_code.as_ref() {
                        api::RESOURCE_ALREADY_EXISTS => {
                            CreateExperimentError::AlreadyExists(name.to_string())
                        }
                        code => CreateExperimentError::Storage(anyhow!(
                            "Unknown error code {}. Message: {}",
                            code,
                            response.message
                        )),
                    }
                } else {
                    CreateExperimentError::Storage(anyhow!(
                        "request failed with status code {}. Body: {}",
                        status,
                        body
                    ))
                }
            });
        }
        let response_body = http_response
            .into_string()
            .context("turning response body into string")?;
        let _response = Response::deserialize_json(&response_body).context("deserializing json")?;
        Ok(self.get_experiment(name).unwrap())
    }

    fn list_experiments(&self) -> Result<Vec<Experiment>, StorageError> {
        use api::list_experiments::Response;
        let endpoint = &self.endpoints.experiments_list;
        let http_response = ureq::get(endpoint).send_string("{}");
        let http_response = validate_response(http_response)?;
        let response_body = http_response
            .into_string()
            .context("turning response body into string")?;
        let response = Response::deserialize_json(&response_body).context("deserializing json")?;
        Ok(response.experiments)
    }

    fn get_experiment(&self, name: &str) -> Result<Experiment, GetExperimentError> {
        use api::get_experiment_by_name::{Request, Response};
        let request = Request::new(name.to_string()).serialize_json();
        let endpoint = &self.endpoints.experiments_get;
        let http_response = ureq::get(endpoint).send_string(&request);
        if http_response.error() {
            let status = http_response.status();
            let body = http_response
                .into_string()
                .context("turning error response into string")?;
            return Err({
                if status == 404 {
                    let response = api::ErrorResponse::deserialize_json(&body)
                        .context("deserializing error body")?;
                    match response.error_code.as_ref() {
                        api::RESOURCE_DOES_NOT_EXIST => {
                            GetExperimentError::DoesNotExist(name.to_string())
                        }
                        code => GetExperimentError::Storage(anyhow!(
                            "Unknown error code {}. Message: {}",
                            code,
                            response.message
                        )),
                    }
                } else {
                    GetExperimentError::Storage(anyhow!(
                        "request failed with status code {}. Body: {}",
                        status,
                        body
                    ))
                }
            });
        }
        let response_body = http_response
            .into_string()
            .context("turning response body into string")?;
        let response = Response::deserialize_json(&response_body).context("deserializing json")?;
        Ok(response.experiment)
    }

    fn create_run(&self, experiment: &str, start_time: u64) -> Result<Run, StorageError> {
        use api::create_run::{Request, Response};
        let request = Request {
            experiment_id: experiment.to_string(),
            user_id: "mlflow-rs".to_string(),
            start_time: start_time.try_into()?,
            tags: Vec::new(),
        };
        let request = request.serialize_json();
        let endpoint = &self.endpoints.runs_create;
        let http_response = ureq::post(endpoint).send_string(&request);
        let http_response = validate_response(http_response)?;
        let response_body = http_response
            .into_string()
            .context("turning response body into string")?;
        let response = Response::deserialize_json(&response_body).context("deserializing json")?;
        Ok(response.run)
    }

    fn terminate_run(&self, run: &str, end_time: u64) -> Result<(), StorageError> {
        use api::update_run::Request;
        let request = Request {
            run_id: run.to_string(),
            status: "FINISHED".to_string(),
            end_time: end_time.try_into()?,
        };
        let request = request.serialize_json();
        let endpoint = &self.endpoints.runs_update;
        let http_response = ureq::post(endpoint).send_string(&request);
        validate_response(http_response)?;
        Ok(())
    }

    fn log_param(&self, run: &str, key: &str, value: &str) -> Result<(), StorageError> {
        use api::log_param::Request;
        let request = Request {
            run_id: run.to_string(),
            key: key.to_string(),
            value: value.to_string(),
        };
        let request = request.serialize_json();
        let endpoint = &self.endpoints.runs_log_param;
        let http_response = ureq::post(endpoint).send_string(&request);
        validate_response(http_response)?;
        Ok(())
    }

    fn log_metric(
        &self,
        run: &str,
        key: &str,
        value: f64,
        time_stamp: u64,
        step: u64,
    ) -> Result<(), StorageError> {
        use api::log_metric::Request;
        let request = Request {
            run_id: run.to_string(),
            key: key.to_string(),
            value,
            timestamp: time_stamp.try_into()?,
            step: step.try_into()?,
        };
        let request = request.serialize_json();
        let endpoint = &self.endpoints.runs_log_metric;
        let http_response = ureq::post(endpoint).send_string(&request);
        validate_response(http_response)?;
        Ok(())
    }

    fn log_batch(&self, run: &str, metrics: &mut Vec<BufferedMetric>) -> Result<(), StorageError> {
        use api::log_batch::Request;
        while !metrics.is_empty() {
            let count = usize::min(metrics.len(), 1000);
            let request = Request {
                run_id: run.to_string(),
                metrics: metrics.drain(..count).map(|m| Metric::from(m)).collect(),
                params: Vec::new(),
            };
            let request = request.serialize_json();
            let endpoint = &self.endpoints.runs_log_batch;
            let http_response = ureq::post(endpoint).send_string(&request);
            validate_response(http_response)?;
        }
        Ok(())
    }
}

mod api {
    use crate::storage::primitive::*;
    use nanoserde::{DeJson, SerJson};

    pub const RESOURCE_ALREADY_EXISTS: &str = "RESOURCE_ALREADY_EXISTS";
    pub const RESOURCE_DOES_NOT_EXIST: &str = "RESOURCE_DOES_NOT_EXIST";
    //pub const INVALID_PARAMETER_VALUE: &str = "INVALID_PARAMETER_VALUE";

    #[derive(DeJson)]
    pub struct ErrorResponse {
        pub error_code: String,
        pub message: String,
    }

    pub mod create_experiment {
        use super::*;

        #[derive(SerJson)]
        pub struct Request {
            pub name: String,
        }

        impl Request {
            pub fn new(name: String) -> Self {
                Request { name }
            }
        }

        #[derive(DeJson)]
        pub struct Response {
            pub experiment_id: String,
        }
    }

    pub mod list_experiments {
        use super::*;

        #[derive(DeJson)]
        pub struct Response {
            pub experiments: Vec<Experiment>,
        }
    }

    pub mod get_experiment_by_name {
        use super::*;

        #[derive(SerJson)]
        pub struct Request {
            pub experiment_name: String,
        }

        impl Request {
            pub fn new(experiment_name: String) -> Self {
                Request { experiment_name }
            }
        }

        #[derive(DeJson)]
        pub struct Response {
            pub experiment: Experiment,
        }
    }

    pub mod create_run {
        use super::*;

        #[derive(SerJson)]
        pub struct Request {
            pub experiment_id: String,
            pub user_id: String,
            pub start_time: i64,
            pub tags: Vec<RunTag>,
        }

        #[derive(DeJson)]
        pub struct Response {
            pub run: Run,
        }
    }

    pub mod update_run {
        use super::*;

        #[derive(SerJson)]
        pub struct Request {
            pub run_id: String,
            pub status: String,
            pub end_time: i64,
        }

        #[derive(DeJson)]
        pub struct Response {
            pub run_info: RunInfo,
        }
    }

    pub mod log_param {
        use super::*;

        #[derive(SerJson)]
        pub struct Request {
            pub run_id: String,
            pub key: String,
            pub value: String,
        }
    }

    pub mod log_metric {
        use super::*;

        #[derive(SerJson)]
        pub struct Request {
            pub run_id: String,
            pub key: String,
            pub value: f64,
            pub timestamp: i64,
            pub step: i64,
        }
    }

    pub mod log_batch {
        use super::*;

        #[derive(SerJson)]
        pub struct Request {
            pub run_id: String,
            pub metrics: Vec<Metric>,
            pub params: Vec<Param>,
        }
    }
}
