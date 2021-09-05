use crate::{
    api::{
        client::{Client, ViewType},
        error::{BatchError, CreateError, DeleteError, GetError, StorageError, UpdateError},
        experiment::Experiment,
        limits,
        run::{Metric, Param, Run, RunData, RunInfo, RunStatus, RunTag},
        search::{PageToken, RunList, Search},
    },
    ExperimentId, RunId,
};
use anyhow::{Context, Error};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fmt::Display,
    io::{Read, Write},
};

#[derive(Deserialize)]
struct RestErrorResponse {
    pub error_code: RestErrorCode,
    pub message: String,
}
#[derive(Debug, Clone, thiserror::Error)]
pub enum RestError {
    #[error("{status} {code}: {message}")]
    Known {
        status: u16,
        code: RestErrorCode,
        message: String,
    },
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

#[derive(PartialEq, Eq)]
enum RestMethod {
    Get,
    Post,
}
impl RestMethod {
    fn handler(&self) -> fn (&str) -> ureq::Request {
        match self {
            Self::Get => ureq::get,
            Self::Post => ureq::post,
        }
    }
}


pub struct Server {
    api_url: String,
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
        }
    }

    fn execute<Ep, Val, Hand, Err>(&mut self, request: Ep, error_handler: Hand) -> Result<Val, Err>
    where
        Ep: Endpoint<Value = Val> + EndpointExt,
        Hand: FnOnce(RestError) -> Err,
        Err: From<anyhow::Error>,
    {
        let url = format!("{}/{}", self.api_url, Ep::PATH);

        let http_response = if Ep::METHOD == RestMethod::Get {
            let query_str = Ep::write_request_query_string(&request).context("serializing request failed")?;
            Ep::METHOD.handler()(&url).query_str(&query_str).call()
        } else {
            let buffer = Ep::write_request_body_string(&request).context("serializing request failed")?;
            Ep::METHOD.handler()(&url).send_string(&buffer)
        };

        if http_response.error() {
            let error = parse_error(http_response);
            Err(error_handler(error))
        } else {
            let response_string = http_response
                .into_string()
                .context("failed to turn response into string")?;
            let response = Ep::read_response_string(&response_string)
                .with_context(|| format!("deserializing response failed:\n{}", &response_string))?;
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
        let request = GetExperiment { experiment_id: id };
        self.execute(request, |error| match error {
            RestError::Known {
                code: RestErrorCode::ResourceDoesNotExist,
                ..
            } => GetError::DoesNotExist(id.as_ref().to_string()),
            _ => GetError::Storage(error.into()),
        })
    }

    fn get_experiment_by_name(&mut self, name: &str) -> Result<Experiment, GetError> {
        let request = GetExperimentByName {
            experiment_name: name,
        };
        self.execute(request, |error| match error {
            RestError::Known {
                code: RestErrorCode::ResourceDoesNotExist,
                ..
            } => GetError::DoesNotExist(name.to_string()),
            _ => GetError::Storage(error.into()),
        })
    }

    fn delete_experiment(&mut self, id: &ExperimentId) -> Result<(), DeleteError> {
        let request = DeleteExperiment { experiment_id: id };
        self.execute(request, |error| match error {
            RestError::Known {
                code: RestErrorCode::ResourceDoesNotExist,
                ..
            } => GetError::DoesNotExist(id.as_ref().to_string()),
            _ => GetError::Storage(error.into()),
        })
    }

    fn update_experiment(
        &mut self,
        id: &ExperimentId,
        new_name: Option<&str>,
    ) -> Result<(), StorageError> {
        let request = UpdateExperiment {
            experiment_id: id,
            new_name,
        };
        self.execute(request, StorageError::from)
    }

    fn create_run(
        &mut self,
        experiment_id: &ExperimentId,
        start_time: i64,
        tags: &[RunTag],
    ) -> Result<Run, StorageError> {
        let request = CreateRun {
            experiment_id,
            start_time,
            tags,
        };
        self.execute(request, StorageError::from)
    }

    fn delete_run(&mut self, id: &RunId) -> Result<(), DeleteError> {
        let request = DeleteRun { run_id: id };
        self.execute(request, |error| match error {
            RestError::Known {
                code: RestErrorCode::ResourceDoesNotExist,
                ..
            } => GetError::DoesNotExist(id.as_ref().to_string()),
            _ => GetError::Storage(error.into()),
        })
    }

    fn get_run(&mut self, id: &RunId) -> Result<Run, GetError> {
        let request = GetRun { run_id: id };
        self.execute(request, |error| match error {
            RestError::Known {
                code: RestErrorCode::ResourceDoesNotExist,
                ..
            } => GetError::DoesNotExist(id.as_ref().to_string()),
            _ => GetError::Storage(error.into()),
        })
    }

    fn update_run(
        &mut self,
        id: &RunId,
        status: RunStatus,
        end_time: i64,
    ) -> Result<RunInfo, UpdateError> {
        let request = UpdateRun {
            run_id: id,
            status,
            end_time,
        };
        self.execute(request, |error| match error {
            RestError::Known {
                code: RestErrorCode::ResourceDoesNotExist,
                ..
            } => UpdateError::DoesNotExist(id.as_ref().to_string()),
            _ => UpdateError::Storage(error.into()),
        })
    }

    fn search_runs(
        &mut self,
        experiment_ids: &[&ExperimentId],
        filter: &str,
        run_view_type: ViewType,
        max_results: i32,
        order_by: Option<&str>,
        page_token: Option<&str>,
    ) -> Result<Search, StorageError> {
        let request = SearchRuns {
            experiment_ids,
            filter,
            run_view_type,
            max_results,
            order_by,
            page_token,
        };
        self.execute(request, StorageError::from)
    }

    fn list_run_infos(
        &mut self,
        experiment: &ExperimentId,
        run_view_type: ViewType,
        max_results: i32,
        order_by: Option<&str>,
        page_token: Option<&str>,
    ) -> Result<RunList, StorageError> {
        let request = ListRunInfos {
            experiment_ids: &[experiment],
            filter: "",
            run_view_type,
            max_results,
            order_by,
            page_token,
        };
        self.execute(request, StorageError::from)
    }

    fn get_metric_history(&mut self, run: &RunId, metric: &str) -> Result<Vec<Metric>, GetError> {
        let request = GetHistory {
            run_id: run,
            metric_key: metric,
        };
        self.execute(request, |error| match error {
            RestError::Known {
                code: RestErrorCode::ResourceDoesNotExist,
                ..
            } => UpdateError::DoesNotExist(run.as_ref().to_string()),
            _ => UpdateError::Storage(error.into()),
        })
    }

    fn log_param(&mut self, run_id: &RunId, key: &str, value: &str) -> Result<(), StorageError> {
        let request = LogParam { run_id, key, value };
        self.execute(request, StorageError::from)
    }

    fn log_metric(
        &mut self,
        run_id: &RunId,
        key: &str,
        value: f64,
        timestamp: i64,
        step: i64,
    ) -> Result<(), StorageError> {
        let request = LogMetric {
            run_id,
            key,
            value,
            timestamp,
            step,
        };
        self.execute(request, StorageError::from)
    }

    fn log_batch(
        &mut self,
        run: &RunId,
        metrics: &[Metric],
        params: &[Param],
        tags: &[RunTag],
    ) -> Result<(), BatchError> {
        if metrics.len() > limits::BATCH_METRICS {
            return Err(BatchError::ToManyMetrics(metrics.len()));
        }
        if params.len() > limits::BATCH_PARAMS {
            return Err(BatchError::ToManyParams(params.len()));
        }
        if tags.len() > limits::BATCH_TAGS {
            return Err(BatchError::ToManyTags(tags.len()));
        }
        let total_len = metrics.len() + params.len() + tags.len();
        if total_len > limits::BATCH_TOTAL {
            return Err(BatchError::ToManyItems(total_len));
        }
        let request = LogBatch {
            run_id: run,
            metrics,
            params,
            tags,
        };
        self.execute(request, |err| BatchError::Storage(err.into()))
    }
}

trait Endpoint {
    const PATH: &'static str;
    const METHOD: RestMethod;

    type Response;
    type Value;

    fn extract(response: Self::Response) -> Self::Value;
}
trait VoidEndpoint {
    const PATH: &'static str;
    const METHOD: RestMethod;
}
trait EndpointExt: Endpoint {
    fn write_request(request: &Self, writer: impl Write) -> Result<(), Error>;
    fn read_response(reader: impl Read) -> Result<Self::Response, Error>;
    fn read_response_string(response: &str) -> Result<Self::Response, Error>;
    fn write_request_body_string(request: &Self) -> Result<String, Error>;
    fn write_request_query_string(request: &Self) -> Result<String, Error>;
}
impl<E> Endpoint for E
where
    E: VoidEndpoint,
{
    const PATH: &'static str = E::PATH;
    const METHOD: RestMethod = E::METHOD;

    type Response = VoidResponse;
    type Value = ();

    fn extract(_response: Self::Response) -> Self::Value {
        ()
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

    fn read_response_string(response: &str) -> Result<Self::Response, Error> {
        let response = serde_json::from_str::<'_, R>(response)?;
        Ok(response)
    }

    fn write_request_body_string(request: &Self) -> Result<String, Error> {
        Ok(serde_json::to_string(request)?)
    }

    fn write_request_query_string(request: &Self) -> Result<String, Error> {
        Ok(serde_qs::to_string(request)?)
    }
}

#[derive(Deserialize)]
struct VoidResponse {}

#[derive(Debug, Clone, Copy, Serialize)]
struct CreateExperiment<'a> {
    pub name: &'a str,
    pub artifact_location: Option<&'a str>,
}
#[derive(Deserialize)]
struct CreateExperimentResponse {
    experiment_id: ExperimentId,
}
impl Endpoint for CreateExperiment<'_> {
    const PATH: &'static str = "2.0/mlflow/experiments/create";
    const METHOD: RestMethod = RestMethod::Post;
    type Response = CreateExperimentResponse;
    type Value = ExperimentId;

    fn extract(response: Self::Response) -> Self::Value {
        response.experiment_id
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct GetExperiment<'a> {
    pub experiment_id: &'a ExperimentId,
}
#[derive(Deserialize)]
struct GetExperimentResponse {
    experiment: Experiment,
}
impl Endpoint for GetExperiment<'_> {
    const PATH: &'static str = "2.0/mlflow/experiments/get";
    const METHOD: RestMethod = RestMethod::Get;
    type Value = Experiment;
    type Response = GetExperimentResponse;

    fn extract(response: Self::Response) -> Self::Value {
        response.experiment
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct UpdateExperiment<'a> {
    pub experiment_id: &'a ExperimentId,
    pub new_name: Option<&'a str>,
}
impl VoidEndpoint for UpdateExperiment<'_> {
    const PATH: &'static str = "2.0/mlflow/experiments/update";
    const METHOD: RestMethod = RestMethod::Post;
}

#[derive(Debug, Clone, Copy, Serialize)]
struct ListExperiments {
    pub view_type: ViewType,
}
#[derive(Deserialize)]
struct ListExperimentsResponse {
    experiments: Vec<Experiment>,
}
impl Endpoint for ListExperiments {
    const PATH: &'static str = "2.0/mlflow/experiments/list";
    const METHOD: RestMethod = RestMethod::Get;
    type Response = ListExperimentsResponse;
    type Value = Vec<Experiment>;

    fn extract(response: Self::Response) -> Self::Value {
        response.experiments
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct GetExperimentByName<'a> {
    pub experiment_name: &'a str,
}
impl Endpoint for GetExperimentByName<'_> {
    const PATH: &'static str = "2.0/mlflow/experiments/get-by-name";
    const METHOD: RestMethod = RestMethod::Get;
    type Value = Experiment;
    type Response = GetExperimentResponse;

    fn extract(response: Self::Response) -> Self::Value {
        response.experiment
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct DeleteExperiment<'a> {
    pub experiment_id: &'a ExperimentId,
}
impl VoidEndpoint for DeleteExperiment<'_> {
    const PATH: &'static str = "2.0/mlflow/experiments/delete";
    const METHOD: RestMethod = RestMethod::Post;
}

#[derive(Debug, Clone, Copy, Serialize)]
struct CreateRun<'a> {
    pub experiment_id: &'a ExperimentId,
    pub start_time: i64,
    pub tags: &'a [RunTag],
}
#[derive(Deserialize)]
struct GetRunResponse {
    run: Run,
}
impl Endpoint for CreateRun<'_> {
    const PATH: &'static str = "2.0/mlflow/runs/create";
    const METHOD: RestMethod = RestMethod::Post;
    type Response = GetRunResponse;
    type Value = Run;

    fn extract(response: Self::Response) -> Self::Value {
        response.run
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct DeleteRun<'a> {
    pub run_id: &'a RunId,
}
impl VoidEndpoint for DeleteRun<'_> {
    const PATH: &'static str = "2.0/mlflow/runs/delete";
    const METHOD: RestMethod = RestMethod::Post;
}

#[derive(Debug, Clone, Copy, Serialize)]
struct GetRun<'a> {
    pub run_id: &'a RunId,
}
impl Endpoint for GetRun<'_> {
    const PATH: &'static str = "";
    const METHOD: RestMethod = RestMethod::Get;
    type Response = GetRunResponse;
    type Value = Run;

    fn extract(response: Self::Response) -> Self::Value {
        response.run
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct LogParam<'a> {
    pub run_id: &'a RunId,
    pub key: &'a str,
    pub value: &'a str,
}
impl VoidEndpoint for LogParam<'_> {
    const PATH: &'static str = "2.0/mlflow/runs/log-parameter";
    const METHOD: RestMethod = RestMethod::Post;
}

#[derive(Debug, Clone, Copy, Serialize)]
struct LogMetric<'a> {
    pub run_id: &'a RunId,
    pub key: &'a str,
    pub value: f64,
    pub timestamp: i64,
    pub step: i64,
}
impl VoidEndpoint for LogMetric<'_> {
    const PATH: &'static str = "2.0/mlflow/runs/log-metric";
    const METHOD: RestMethod = RestMethod::Post;
}

#[derive(Debug, Clone, Copy, Serialize)]
struct UpdateRun<'a> {
    pub run_id: &'a RunId,
    pub status: RunStatus,
    pub end_time: i64,
}
#[derive(Deserialize)]
struct UpdateRunResponse {
    run_info: RunInfo,
}
impl Endpoint for UpdateRun<'_> {
    const PATH: &'static str = "2.0/mlflow/runs/update";
    const METHOD: RestMethod = RestMethod::Post;
    type Response = UpdateRunResponse;
    type Value = RunInfo;

    fn extract(response: Self::Response) -> Self::Value {
        response.run_info
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct LogBatch<'a> {
    pub run_id: &'a RunId,
    pub metrics: &'a [Metric<'a>],
    pub params: &'a [Param],
    pub tags: &'a [RunTag],
}
impl VoidEndpoint for LogBatch<'_> {
    const PATH: &'static str = "2.0/mlflow/runs/log-batch";
    const METHOD: RestMethod = RestMethod::Post;
}

#[derive(Debug, Clone, Copy, Serialize)]
struct SearchRuns<'a> {
    pub experiment_ids: &'a [&'a ExperimentId],
    pub filter: &'a str,
    pub run_view_type: ViewType,
    pub max_results: i32,
    pub order_by: Option<&'a str>,
    pub page_token: Option<&'a str>,
}
impl Endpoint for SearchRuns<'_> {
    const PATH: &'static str = "";
    const METHOD: RestMethod = RestMethod::Post;
    type Response = Search;
    type Value = Search;

    fn extract(response: Self::Response) -> Self::Value {
        response
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct ListRunInfos<'a> {
    pub experiment_ids: &'a [&'a ExperimentId],
    pub filter: &'a str,
    pub run_view_type: ViewType,
    pub max_results: i32,
    pub order_by: Option<&'a str>,
    pub page_token: Option<&'a str>,
}
#[derive(Deserialize)]
struct ListRunInfosRun {
    info: RunInfo,

    #[allow(dead_code)]
    #[serde(default, skip)]
    data: RunData,
}
#[derive(Deserialize)]
struct ListRunInfosResponse {
    pub runs: Vec<ListRunInfosRun>,
    pub next_page_token: PageToken,
}
impl Endpoint for ListRunInfos<'_> {
    const PATH: &'static str = SearchRuns::PATH;
    const METHOD: RestMethod = SearchRuns::METHOD;
    type Response = ListRunInfosResponse;
    type Value = RunList;

    fn extract(response: Self::Response) -> Self::Value {
        RunList {
            runs: response.runs.into_iter().map(|r| r.info).collect(),
            page_token: response.next_page_token,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
struct GetHistory<'a> {
    pub run_id: &'a RunId,
    pub metric_key: &'a str,
}
#[derive(Deserialize)]
struct GetHistoryResponse {
    metrics: Vec<Metric<'static>>,
}
impl Endpoint for GetHistory<'_> {
    const PATH: &'static str = "2.0/mlflow/metrics/get-history";
    const METHOD: RestMethod = RestMethod::Get;
    type Response = GetHistoryResponse;
    type Value = Vec<Metric<'static>>;

    fn extract(response: Self::Response) -> Self::Value {
        response.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::GetExperimentResponse;

    #[test]
    fn parse_get_experiment_response() {
        let response = r#"
        {
            "experiment": {
                "experiment_id": "1",
                "name": "T1",
                "artifact_location": "./mlruns/1",
                "lifecycle_stage": "active"
            }
        }
        "#;
        let parsed = serde_json::from_str::<GetExperimentResponse>(response).unwrap();
        assert_eq!(parsed.experiment.experiment_id.as_ref(), "1");
    }
}
