# mlflow-rs

Rust library providing access to the MLflow REST API.

This is **not** an official project.

# Example

```rust
fn main() {
    let client = Client::for_server("http://127.0.0.1:5000/api");
    let experiment = client.create_experiment("My Experiment");

    for i in 0..3 {
        println!("Executing run {}", i);
        let run = experiment.create_run();

        run.log_param("i", &format!("{}", i));
        run.log_param("constant", "42");

        let mut rng = WyRand::new_seed(i.into());
        for s in 0..10 {
            let int: f64 = rng.generate::<u16>().into();
            let max: f64 = std::u16::MAX.into();
            let value = int / max;
            run.log_metric("rand", value, timestamp(), s);
        }
        run.terminate();
    }
}
```

# State

The following parts of the API are implemented:

- [x] Experiments
    - [x] Create
    - [x] Read
    - [x] Update
- [x] Runs
    - [x] Create
    - [x] Read
    - [x] Update
    - [x] Search
- [ ] Logging
    - [x] Parameters
    - [x] Metrics
    - [ ] Artifacts
- [ ] Models
    - [ ] Create
    - [ ] Read
    - [ ] Update
- [x] Tags
