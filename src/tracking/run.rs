use crate::{
    api::{
        error::StorageError,
        limits,
        run::{Metric, Param, Run, RunTag},
    },
    timestamp, Client, ExperimentId,
};

/// A MLflow Run.
///
/// This can be created using [`Experiment::create_run`].
///
/// It allows logging [parameters][self::Run::log_param()] and [metrics][self::Run::log_metric()].
pub struct TrackingRun<'b> {
    start_time: i64,
    param_buffer: Vec<Param>,
    tag_buffer: Vec<RunTag>,
    metric_buffer: Vec<Vec<Metric<'b>>>,
}

impl<'b> TrackingRun<'b> {
    pub fn new() -> Self {
        TrackingRun {
            start_time: timestamp(),
            param_buffer: Vec::new(),
            tag_buffer: Vec::new(),
            metric_buffer: vec![Vec::with_capacity(limits::BATCH_METRICS)],
        }
    }

    pub fn log_param(&mut self, param: Param) {
        assert!(
            self.param_buffer.len() < limits::BATCH_PARAMS,
            "TrackingRun supports only up to 100 params for now"
        );
        self.param_buffer.push(param);
    }

    pub fn log_tag(&mut self, tag: RunTag) {
        assert!(
            self.tag_buffer.len() < limits::BATCH_TAGS,
            "TrackingRun supports only up to 100 tags for now"
        );
        self.tag_buffer.push(tag);
    }

    pub fn log_metric(&mut self, metric: Metric<'b>) {
        if self.metric_buffer.last().unwrap().len() == limits::BATCH_METRICS {
            self.metric_buffer.push(Vec::with_capacity(limits::BATCH_METRICS));
        }
        self.metric_buffer.last_mut().unwrap().push(metric);
    }

    pub fn submit(self, client: &mut dyn Client, experiment: &ExperimentId) -> Result<Run, StorageError> {
        let mut run = client.create_run(experiment, self.start_time, &[])?;
        let id = &run.info.run_id.clone();
        client.log_batch(id, &[], &self.param_buffer, &self.tag_buffer)?;
        for buffer in &self.metric_buffer {
            client.log_batch(id, buffer, &[], &[])?;
        }
        run.info = client.update_run(id, crate::api::run::RunStatus::Finished, timestamp())?;
        Ok(run)
    }
}
