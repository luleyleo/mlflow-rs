use crate::{
    api::{
        client::{Client, ViewType},
        error::{BatchError, CreateError, DeleteError, GetError, StorageError, UpdateError},
        experiment::Experiment,
        run::{Metric, Param, Run, RunInfo, RunStatus, RunTag},
    },
    ExperimentId,
};
use anyhow::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fmt::Display,
    io::{Read, Write},
};

#[derive(Deserialize)]
pub struct RestErrorResponse {
    pub error_code: RestErrorCode,
    pub message: String,
}
#[derive(Debug, Clone, thiserror::Error)]
pub enum RestError {
    #[error("{status} {code}: {message}")]
    Known { status: u16, code: RestErrorCode, message: String },
    #[error("Unknown {status} error:\n{body}")]
    Unknown { status: u16, body: String },
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(from = "&str")]
pub enum RestErrorCode {
    ResourceAlreadyExists,
    ResourceDoesNotExist,
    InvalidParameterValue,
    Unknown(String),
}
impl From<&str> for RestErrorCode {
    fn from(value: &str) -> Self {
        match value {
            "RESOURCE_ALREADY_EXISTS" => RestErrorCode::ResourceAlreadyExists,
            "RESOURCE_DOES_NOT_EXIST" => RestErrorCode::ResourceDoesNotExist,
            "INVALID_PARAMETER_VALUE" => RestErrorCode::InvalidParameterValue,
            _ => return RestErrorCode::Unknown(value.to_owned()),
        }
    }
}
impl Display for RestErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Server {
    api_url: String,
    buffer: String,
}

fn parse_error(response: ureq::Response) -> RestError {
    let status = response.status();
    let body = response
        .into_string()
        .unwrap_or_else(|_| "Could not turn error body into String.".to_string());

    let response = serde_json::from_str::<RestErrorResponse>(&body).ok();
    if let Some(response) = response {
        RestError::Known {
            status,
            code: response.error_code,
            message: response.message,
        }
    } else {
        RestError::Unknown { status, body }
    }
}

impl Server {
    pub fn new(api_url: impl Into<String>) -> Self {
        Server {
            api_url: api_url.into(),
            buffer: String::new(),
        }
    }

    fn execute<Ep, Val, Hand, Err>(&mut self, request: Ep, error_handler: Hand) -> Result<Val, Err>
    where
        Ep: Endpoint<Value = Val> + EndpointExt,
        Hand: FnOnce(RestError) -> Err,
        Err: From<anyhow::Error>,
    {
        let url = format!("{}/{}", self.api_url, Ep::PATH);
        self.buffer.clear();
        Ep::write_request_string(&request, &mut self.buffer)?;
        let http_response = ureq::get(&url).send_string(&self.buffer);
        if http_response.error() {
            let error = parse_error(http_response);
            Err(error_handler(error))
        } else {
            let response = Ep::read_response(http_response.into_reader())?;
            let value = Ep::extract(response);
            Ok(value)
        }
    }
}

#[allow(unused_variables)]
impl Client for Server {
    fn create_experiment(&mut self, name: &str) -> Result<ExperimentId, CreateError> {
        let request = CreateExperiment {
            name,
            artifact_location: None,
        };
        self.execute(request, |error| match error {
            RestError::Known {
                code: RestErrorCode::ResourceAlreadyExists,
                ..
            } => CreateError::AlreadyExists(name.to_string()),
            _ => CreateError::Storage(error.into()),
        })
    }

    fn list_experiments(&mut self, view_type: ViewType) -> Result<Vec<Experiment>, StorageError> {
        let request = ListExperiments { view_type };
        self.execute(request, StorageError::from)
    }

    fn get_experiment(&mut self, id: &ExperimentId) -> Result<Experiment, GetError> {
        todo!()
    }

    fn get_experiment_by_name(&mut self, name: &str) -> Result<Experiment, GetError> {
        todo!()
    }

    fn delete_experiment(&mut self, id: &ExperimentId) -> Result<(), DeleteError> {
        todo!()
    }

    fn update_experiment(&mut self, id: &ExperimentId, new_name: Option<&str>) -> Result<(), StorageError> {
        todo!()
    }

    fn create_run(&mut self, experiment: &ExperimentId, start_time: i64, tags: &[RunTag]) -> Result<Run, StorageError> {
        todo!()
    }

    fn delete_run(&mut self, id: &crate::RunId) -> Result<(), DeleteError> {
        todo!()
    }

    fn get_run(&mut self, id: &crate::RunId) -> Result<Run, GetError> {
        todo!()
    }

    fn update_run(&mut self, id: &crate::RunId, status: RunStatus, end_time: i64) -> Result<RunInfo, UpdateError> {
        todo!()
    }

    fn log_param(&mut self, run: &crate::RunId, key: &str, value: &str) -> Result<(), StorageError> {
        todo!()
    }

    fn log_metric(&mut self, run: &crate::RunId, key: &str, value: f64, timestamp: i64, step: i64) -> Result<(), StorageError> {
        todo!()
    }

    fn log_batch(&mut self, run: &crate::RunId, metrics: &[Metric], params: &[Param], tags: &[RunTag]) -> Result<(), BatchError> {
        todo!()
    }
}

trait Endpoint {
    const PATH: &'static str;
    type Response;
    type Value;

    fn extract(response: Self::Response) -> Self::Value;
}
trait SimpleEndpoint {
    const PATH: &'static str;
    type Value;
}
trait EndpointExt: Endpoint {
    fn write_request(request: &Self, writer: impl Write) -> Result<(), Error>;
    fn read_response(reader: impl Read) -> Result<Self::Response, Error>;
    fn write_request_string(request: &Self, buffer: &mut String) -> Result<(), Error>;
}
impl<E, V> Endpoint for E
where
    E: SimpleEndpoint<Value = V>,
{
    const PATH: &'static str = E::PATH;

    type Response = V;
    type Value = V;

    fn extract(response: Self::Response) -> Self::Value {
        response
    }
}
impl<P, R, V> EndpointExt for P
where
    P: Serialize,
    R: DeserializeOwned,
    P: Endpoint<Response = R, Value = V>,
{
    fn write_request(request: &Self, writer: impl Write) -> Result<(), Error> {
        serde_json::to_writer(writer, &request)?;
        Ok(())
    }

    fn read_response(reader: impl Read) -> Result<Self::Response, Error> {
        let response = serde_json::from_reader::<_, R>(reader)?;
        Ok(response)
    }

    fn write_request_string(request: &Self, buffer: &mut String) -> Result<(), Error> {
        // This is similar to what Serde does internally.
        // See https://docs.serde.rs/src/serde_json/ser.rs.html#2219-2229
        let bytes = unsafe { buffer.as_bytes_mut() };
        let cursor = std::io::Cursor::new(bytes);
        Self::write_request(request, cursor)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct CreateExperiment<'a> {
    pub name: &'a str,
    pub artifact_location: Option<&'a str>,
}
#[derive(Deserialize)]
struct CreateExperimentResponse {
    experiment_id: ExperimentId,
}
impl Endpoint for CreateExperiment<'_> {
    const PATH: &'static str = "2.0/mlflow/experiments/create";
    type Response = CreateExperimentResponse;
    type Value = ExperimentId;

    fn extract(response: Self::Response) -> Self::Value {
        response.experiment_id
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct GetExperiment<'a> {
    pub experiment_id: &'a str,
}
impl SimpleEndpoint for GetExperiment<'_> {
    const PATH: &'static str = "2.0/mlflow/experiments/get";
    type Value = Experiment;
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ListExperiments {
    view_type: ViewType,
}
#[derive(Deserialize)]
struct ListExperimentsResponse {
    experiments: Vec<Experiment>,
}
impl Endpoint for ListExperiments {
    const PATH: &'static str = "2.0/mlflow/experiments/list";
    type Response = ListExperimentsResponse;
    type Value = Vec<Experiment>;

    fn extract(response: Self::Response) -> Self::Value {
        response.experiments
    }
}
