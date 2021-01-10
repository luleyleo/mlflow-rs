use crate::{api::experiment::Experiment, ExperimentId};
use anyhow::Error;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{Read, Write};

trait Get {
    const ENDPOINT: &'static str;
    type Resource;

    fn write_request(&self, writer: impl Write) -> Result<(), Error>;
    fn parse_response(&self, reader: impl Read) -> Result<Self::Resource, Error>;

    fn write_request_string(&self, buffer: &mut String) -> Result<(), Error> {
        // This is similar to what Serde does internally.
        // See https://docs.serde.rs/src/serde_json/ser.rs.html#2219-2229
        let bytes = unsafe { buffer.as_bytes_mut() };
        let cursor = std::io::Cursor::new(bytes);
        self.write_request(cursor)?;
        Ok(())
    }
}

trait Post {
    const ENDPOINT: &'static str;
    type Response;
    type Value;

    fn extract(&self, response: Self::Response) -> Self::Value;
}
trait PostExt {
    type Response;

    fn write_request(&self, writer: impl Write) -> Result<(), Error>;
    fn read_response(&self, reader: impl Read) -> Result<Self::Response, Error>;
    fn write_request_string(&self, buffer: &mut String) -> Result<(), Error>;
}

impl<P, R, V> PostExt for P
where
    P: Serialize,
    R: DeserializeOwned,
    P: Post<Response = R, Value = V>,
{
    type Response = R;

    fn write_request(&self, writer: impl Write) -> Result<(), Error> {
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    fn read_response(&self, reader: impl Read) -> Result<Self::Response, Error> {
        let response = serde_json::from_reader::<_, R>(reader)?;
        Ok(response)
    }

    fn write_request_string(&self, buffer: &mut String) -> Result<(), Error> {
        // This is similar to what Serde does internally.
        // See https://docs.serde.rs/src/serde_json/ser.rs.html#2219-2229
        let bytes = unsafe { buffer.as_bytes_mut() };
        let cursor = std::io::Cursor::new(bytes);
        self.write_request(cursor)?;
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
impl Post for CreateExperiment<'_> {
    const ENDPOINT: &'static str = "2.0/mlflow/experiments/create";
    type Response = CreateExperimentResponse;
    type Value = ExperimentId;

    fn extract(&self, response: Self::Response) -> Self::Value {
        response.experiment_id
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct GetExperiment<'a> {
    pub experiment_id: &'a str,
}
impl Get for GetExperiment<'_> {
    const ENDPOINT: &'static str = "2.0/mlflow/experiments/get";
    type Resource = Experiment;

    fn write_request(&self, writer: impl Write) -> Result<(), Error> {
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    fn parse_response(&self, reader: impl Read) -> Result<Self::Resource, Error> {
        let experiment = serde_json::from_reader(reader)?;
        Ok(experiment)
    }
}
