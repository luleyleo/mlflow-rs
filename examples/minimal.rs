use mlflow::{Client, backend::rest::Server, tracking::TrackingRun};
use nanorand::{RNG, WyRand};

fn main() {
    const EXPERIMENT: &str = "My Experiment";
    let mut client = Server::new("http://127.0.0.1:5000/api");
    let experiment = client.get_experiment_by_name(EXPERIMENT)
        .map(|experiment| experiment.experiment_id)
        .or_else(|_| client.create_experiment(EXPERIMENT))
        .expect("Could neither get nor create the experiment");

    for i in 0..3 {
        println!("Executing run {}", i);
        let mut run = TrackingRun::new();
        run.log_param("i", &format!("{}", i));
        run.log_param("constant", "42");
        let mut rng = WyRand::new_seed(i);
        for s in 0..10 {
            let int: f64 = rng.generate::<u16>().into();
            let max: f64 = std::u16::MAX.into();
            let value = int / max;
            run.log_metric("rand", value, s);
        }
        run.submit(&mut client, &experiment)
            .expect("Could not submit the run");
    }
}
