use crate::storage::{self, errors, primitive};
use anyhow::{anyhow, Context};
use errors::{CreateExperimentError, GetExperimentError, StorageError};
use std::{convert::TryInto, sync::Arc};

pub struct ClientStorage {
    server: Server,
}

pub struct ExperimentStorage {
    server: Server,
}

pub struct RunStorage {
    server: Server,
    buffer: Vec<BufferedMetric>,
}

#[derive(Clone)]
struct Server {
    endpoints: Arc<Endpoints>,
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

#[derive(serde::Serialize)]
pub struct BufferedMetric {
    pub name: &'static str,
    pub value: f64,
    pub timestamp: u64,
    pub step: u64,
}

impl Server {
    pub fn new(url: &str) -> Self {
        Server {
            endpoints: Arc::new(Endpoints::new(url)),
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

impl ClientStorage {
    pub fn new(url: &str) -> Self {
        ClientStorage {
            server: Server::new(url),
        }
    }
}
impl storage::ClientStorage for ClientStorage {
    fn create_experiment(
        &mut self,
        name: &str,
    ) -> Result<crate::Experiment, CreateExperimentError> {
        let primitive = self.server.create_experiment(name)?;
        let storage = ExperimentStorage::new(self.server.clone());
        Ok(crate::Experiment::new(Box::new(storage), primitive))
    }

    fn list_experiments(&mut self) -> Result<Vec<crate::Experiment>, StorageError> {
        let primitive = self.server.list_experiments()?;
        let experiments: Vec<crate::Experiment> = primitive
            .into_iter()
            .map(|primitive| {
                let storage = ExperimentStorage::new(self.server.clone());
                crate::Experiment::new(Box::new(storage), primitive)
            })
            .collect();
        Ok(experiments)
    }

    fn get_experiment(&mut self, name: &str) -> Result<crate::Experiment, GetExperimentError> {
        let primitive = self.server.get_experiment(name)?;
        let storage = ExperimentStorage::new(self.server.clone());
        Ok(crate::Experiment::new(Box::new(storage), primitive))
    }
}

impl ExperimentStorage {
    fn new(server: Server) -> Self {
        ExperimentStorage { server }
    }
}
impl storage::ExperimentStorage for ExperimentStorage {
    fn create_run(
        &mut self,
        experiment: &str,
        start_time: u64,
    ) -> Result<crate::Run, StorageError> {
        let primitive = self.server.create_run(experiment, start_time)?;
        let storage = RunStorage::new(self.server.clone());
        Ok(crate::Run::new(Box::new(storage), primitive))
    }
}

impl RunStorage {
    fn new(server: Server) -> Self {
        RunStorage {
            server,
            buffer: Vec::with_capacity(1000),
        }
    }
}
impl storage::RunStorage for RunStorage {
    fn log_param(&mut self, run: &str, key: &str, value: &str) -> Result<(), StorageError> {
        self.server.log_param(run, key, value)
    }

    fn log_metric(
        &mut self,
        _run: &str,
        key: &'static str,
        value: f64,
        timestamp: u64,
        step: u64,
    ) -> Result<(), StorageError> {
        self.buffer.push(BufferedMetric {
            name: key,
            value,
            timestamp,
            step,
        });
        Ok(())
    }

    fn terminate(&mut self, run: &str, end_time: u64) -> Result<(), StorageError> {
        self.server.log_batch(run, &mut self.buffer)?;
        self.server.terminate_run(run, end_time)
    }
}

impl Server {
    fn create_experiment(
        &self,
        name: &str,
    ) -> Result<primitive::Experiment, CreateExperimentError> {
        use api::create_experiment::{Request, Response};

        let request = serde_json::to_string(&Request { name }).context("serializing request")?;

        let endpoint = &self.endpoints.experiments_create;
        let http_response = ureq::post(endpoint).send_string(&request);

        if http_response.error() {
            let status = http_response.status();
            let body = http_response
                .into_string()
                .context("turning error response into string")?;
            return Err({
                if status == 400 {
                    let response: api::ErrorResponse =
                        serde_json::from_str(&body).context("deserializing error body")?;
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
        let _response: Response =
            serde_json::from_str(&response_body).context("deserializing json")?;
        // TODO: use get_experiment_by_id
        Ok(self.get_experiment(name).unwrap())
    }

    fn list_experiments(&self) -> Result<Vec<primitive::Experiment>, StorageError> {
        use api::list_experiments::Response;
        let endpoint = &self.endpoints.experiments_list;
        let http_response = ureq::get(endpoint).send_string("{}");
        let http_response = validate_response(http_response)?;
        let response_body = http_response
            .into_string()
            .context("turning response body into string")?;
        let response =
            serde_json::from_str::<Response>(&response_body).context("deserializing json")?;
        Ok(response.experiments)
    }

    fn get_experiment(
        &self,
        experiment_name: &str,
    ) -> Result<primitive::Experiment, GetExperimentError> {
        use api::get_experiment_by_name::{Request, Response};
        let request =
            serde_json::to_string(&Request { experiment_name }).context("serializing request")?;
        let endpoint = &self.endpoints.experiments_get;
        let http_response = ureq::get(endpoint).send_string(&request);

        if http_response.error() {
            let status = http_response.status();
            let body = http_response
                .into_string()
                .context("turning error response into string")?;
            return Err({
                if status == 404 {
                    let response = serde_json::from_str::<api::ErrorResponse>(&body)
                        .context("deserializing error body")?;
                    match response.error_code.as_ref() {
                        api::RESOURCE_DOES_NOT_EXIST => {
                            GetExperimentError::DoesNotExist(experiment_name.to_string())
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
        let response: Response =
            serde_json::from_str(&response_body).context("deserializing json")?;
        Ok(response.experiment)
    }

    fn create_run(
        &self,
        experiment: &str,
        start_time: u64,
    ) -> Result<primitive::Run, StorageError> {
        use api::create_run::{Request, Response};
        let request = Request {
            experiment_id: experiment,
            user_id: "mlflow-rs",
            start_time: start_time.try_into()?,
            tags: &[],
        };
        let request = serde_json::to_string(&request).context("serializing request")?;
        let endpoint = &self.endpoints.runs_create;
        let http_response = ureq::post(endpoint).send_string(&request);
        let http_response = validate_response(http_response)?;
        let response_body = http_response
            .into_string()
            .context("turning response body into string")?;
        let response =
            serde_json::from_str::<Response>(&response_body).context("deserializing json")?;
        Ok(response.run)
    }

    fn terminate_run(&self, run: &str, end_time: u64) -> Result<(), StorageError> {
        use api::update_run::Request;
        let request = Request {
            run_id: run,
            status: "FINISHED",
            end_time: end_time.try_into()?,
        };
        let request = serde_json::to_string(&request).context("serializing request")?;
        let endpoint = &self.endpoints.runs_update;
        let http_response = ureq::post(endpoint).send_string(&request);
        validate_response(http_response)?;
        Ok(())
    }

    fn log_param(&self, run: &str, key: &str, value: &str) -> Result<(), StorageError> {
        use api::log_param::Request;
        let request = Request {
            run_id: run,
            key: key,
            value: value,
        };
        let request = serde_json::to_string(&request).context("serializing request")?;
        let endpoint = &self.endpoints.runs_log_param;
        let http_response = ureq::post(endpoint).send_string(&request);
        validate_response(http_response)?;
        Ok(())
    }

    #[allow(dead_code)]
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
            run_id: run,
            key,
            value,
            timestamp: time_stamp.try_into()?,
            step: step.try_into()?,
        };
        let request = serde_json::to_string(&request).context("serializing request")?;
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
                run_id: run,
                metrics: &metrics[..count],
                params: &[],
            };
            let request = serde_json::to_string(&request).context("serializing request")?;
            let endpoint = &self.endpoints.runs_log_batch;
            let http_response = ureq::post(endpoint).send_string(&request);
            validate_response(http_response)?;
        }
        Ok(())
    }
}

mod api {
    use crate::storage::primitive::*;
    use serde::{Deserialize, Serialize};

    use super::BufferedMetric;

    pub const RESOURCE_ALREADY_EXISTS: &str = "RESOURCE_ALREADY_EXISTS";
    pub const RESOURCE_DOES_NOT_EXIST: &str = "RESOURCE_DOES_NOT_EXIST";
    //pub const INVALID_PARAMETER_VALUE: &str = "INVALID_PARAMETER_VALUE";

    impl Into<Metric> for BufferedMetric {
        fn into(self) -> Metric {
            Metric {
                key: self.name.to_string(),
                value: self.value,
                timestamp: self.timestamp as i64,
                step: self.step as i64,
            }
        }
    }

    #[derive(Deserialize)]
    pub struct ErrorResponse<'a> {
        pub error_code: &'a str,
        pub message: &'a str,
    }

    pub mod create_experiment {
        use super::*;

        #[derive(Serialize)]
        pub struct Request<'a> {
            pub name: &'a str,
        }

        #[derive(Deserialize)]
        pub struct Response<'a> {
            pub experiment_id: &'a str,
        }
    }

    pub mod list_experiments {
        use super::*;

        #[derive(Deserialize)]
        pub struct Response {
            pub experiments: Vec<Experiment>,
        }
    }

    pub mod get_experiment_by_name {
        use super::*;

        #[derive(Serialize)]
        pub struct Request<'a> {
            pub experiment_name: &'a str,
        }

        #[derive(Deserialize)]
        pub struct Response {
            pub experiment: Experiment,
        }
    }

    pub mod create_run {
        use super::*;

        #[derive(Serialize)]
        pub struct Request<'a> {
            pub experiment_id: &'a str,
            pub user_id: &'a str,
            pub start_time: i64,
            pub tags: &'a [RunTag],
        }

        #[derive(Deserialize)]
        pub struct Response {
            pub run: Run,
        }
    }

    pub mod update_run {
        use super::*;

        #[derive(Serialize)]
        pub struct Request<'a> {
            pub run_id: &'a str,
            pub status: &'a str,
            pub end_time: i64,
        }

        #[derive(Deserialize)]
        pub struct Response {
            pub run_info: RunInfo,
        }
    }

    pub mod log_param {
        use super::*;

        #[derive(Serialize)]
        pub struct Request<'a> {
            pub run_id: &'a str,
            pub key: &'a str,
            pub value: &'a str,
        }
    }

    pub mod log_metric {
        use super::*;

        #[derive(Serialize)]
        pub struct Request<'a> {
            pub run_id: &'a str,
            pub key: &'a str,
            pub value: f64,
            pub timestamp: i64,
            pub step: i64,
        }
    }

    pub mod log_batch {
        use super::*;

        #[derive(Serialize)]
        pub struct Request<'a> {
            pub run_id: &'a str,
            pub metrics: &'a [BufferedMetric],
            pub params: &'a [Param],
        }
    }
}
