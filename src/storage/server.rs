use std::convert::TryInto;

use crate::storage::errors;
use crate::storage::primitive::{Experiment, Run};
use crate::Id;
use anyhow::Context;
use errors::{CreateExperimentError, StorageError};
use nanoserde::{DeJson, SerJson};

pub struct Storage {
    endpoints: Endpoints,
}

struct Endpoints {
    pub experiments_create: String,
    pub experiments_list: String,
    pub experiments_get: String,
    pub runs_create: String,
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
        }
    }
}

fn validate_response(response: &ureq::Response) -> Result<(), StorageError> {
    if response.error() {
        return Err(anyhow::anyhow!(
            "request failed with status code {}",
            response.status()
        ));
    }
    Ok(())
}

impl super::Storage for Storage {
    fn create_experiment(&self, name: &str) -> Result<Experiment, CreateExperimentError> {
        use api::create_experiment::{Request, Response};
        let request = Request::new(name.to_string()).serialize_json();
        let endpoint = &self.endpoints.experiments_create;
        let http_response = ureq::post(endpoint).send_string(&request);
        validate_response(&http_response)?;
        let response_body = http_response
            .into_string()
            .context("turning response body into string")?;
        let response = Response::deserialize_json(&response_body).context("deserializing json")?;
        Ok(self.get_experiment(&response.id).unwrap())
    }

    fn list_experiments(&self) -> Result<Vec<Experiment>, StorageError> {
        use api::list_experiments::Response;
        let endpoint = &self.endpoints.experiments_list;
        let http_response = ureq::get(endpoint).send_string("{}");
        validate_response(&http_response)?;
        let response_body = http_response
            .into_string()
            .context("turning response body into string")?;
        let response = Response::deserialize_json(&response_body).context("deserializing json")?;
        Ok(response.experiments)
    }

    fn get_experiment(&self, name: &str) -> Result<Experiment, errors::GetExperimentError> {
        use api::get_experiment_by_name::{Request, Response};
        let request = Request::new(name.to_string()).serialize_json();
        let endpoint = &self.endpoints.experiments_create;
        let http_response = ureq::post(endpoint).send_string(&request);
        validate_response(&http_response)?;
        let response_body = http_response
            .into_string()
            .context("turning response body into string")?;
        let response = Response::deserialize_json(&response_body).context("deserializing json")?;
        Ok(response.experiment)
    }

    fn create_run(&self, experiment: &Id, start_time: u64) -> Result<Run, StorageError> {
        use api::create_run::{Request, Response};
        let request = Request {
            experiment_id: experiment.to_string(),
            user_id: "mlflow-rs".to_string(),
            start_time: start_time.try_into()?,
            tags: Vec::new(),
        };
        let request = request.serialize_json();
        let endpoint = &self.endpoints.experiments_create;
        let http_response = ureq::post(endpoint).send_string(&request);
        validate_response(&http_response)?;
        let response_body = http_response
            .into_string()
            .context("turning response body into string")?;
        let response = Response::deserialize_json(&response_body).context("deserializing json")?;
        Ok(response.run)
    }

    fn terminate_run(&self, run: &Id, end_time: u64) -> Result<(), StorageError> {
        use api::update_run::Request;
        let request = Request {
            run_id: run.to_string(),
            status: "FINISHED".to_string(),
            end_time: end_time.try_into()?,
        };
        let request = request.serialize_json();
        let endpoint = &self.endpoints.experiments_create;
        let http_response = ureq::post(endpoint).send_string(&request);
        validate_response(&http_response)?;
        Ok(())
    }

    fn log_param(&self, run: &Id, key: &str, value: &str) -> Result<(), StorageError> {
        use api::log_param::Request;
        let request = Request {
            run_id: run.to_string(),
            key: key.to_string(),
            value: value.to_string(),
        };
        let request = request.serialize_json();
        let endpoint = &self.endpoints.experiments_create;
        let http_response = ureq::post(endpoint).send_string(&request);
        validate_response(&http_response)?;
        Ok(())
    }

    fn log_metric(
        &self,
        run: &Id,
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
        let endpoint = &self.endpoints.experiments_create;
        let http_response = ureq::post(endpoint).send_string(&request);
        validate_response(&http_response)?;
        Ok(())
    }
}

mod api {
    use crate::storage::primitive::*;
    use nanoserde::{DeJson, SerJson};

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
            pub id: String,
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
}
