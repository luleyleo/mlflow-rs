
pub struct Client {}

pub struct Experiment<'a> {
    client: &'a Client,
    name: String,
}

pub struct Run<'a> {
    experiment: &'a Experiment<'a>,
}

impl Client {
    pub fn create_experiment(&self, name: &str) -> Experiment {
        Experiment {
            client: self,
            name: name.to_string(),
        }
    }
    pub fn get_experiment(&self, name: &str) -> Option<Experiment> {
        None
    }
    pub fn list_experiments(&self) -> Vec<Experiment> {
        Vec::new()
    }
}

impl Experiment<'_> {
    pub fn create_run(&self) -> Run {
        Run {
            experiment: self,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Run<'_> {
    pub fn log_param(&self, name: &str, value: &str) {}
    pub fn log_metric(&self, name: &str, value: f64) {}
    pub fn terminate(self) {}
}
