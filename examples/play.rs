use anyhow::Result;
use mlflow::{backend, tracking::TrackingRun, Client};
use nanorand::{WyRand, RNG};

struct Args {
    experiment: String,
    create: bool,
    runs: u32,
}

impl Args {
    pub fn from_env() -> Result<Self> {
        let mut args = pico_args::Arguments::from_env();
        Ok(Args {
            experiment: args.value_from_str(["-e", "--experiment"])?,
            create: args.contains(["-c", "--create"]),
            runs: args.opt_value_from_str(["-r", "--runs"])?.unwrap_or(1),
        })
    }
}

fn main() -> Result<()> {
    let args = Args::from_env()?;
    let mut client = backend::rest::Server::new("http://127.0.0.1:5000/api");
    let experiment = if args.create {
        use mlflow::api::error::{CreateError, GetError};
        let experiment_id = client.create_experiment(&args.experiment);
        match experiment_id.as_ref() {
            Ok(experiment_id) => {
                println!("Experiment with id {} was created successfully!", experiment_id.as_ref());
            }
            Err(CreateError::AlreadyExists(name)) => {
                println!("The experiment {} already exists.", name);
                println!("Run again without the -c or --create flag to fetch the existing experiment.");
                return Ok(());
            }
            Err(CreateError::Storage(err)) => {
                println!("Failed to create experiment:\n {}", err);
                return Ok(());
            }
        }
        let experiment = client.get_experiment(&experiment_id.unwrap());
        match experiment {
            Ok(experiment) => {
                println!("Experiment {} was requested successfully!", experiment.name);
                experiment
            }
            Err(GetError::DoesNotExist(name)) => {
                println!("The experiment {} does not exists.", name);
                println!("Run again with the -c or --create flag to create a new experiment.");
                return Ok(());
            }
            Err(GetError::Storage(err)) => {
                println!("Failed to get experiment:\n {}", err);
                return Ok(());
            }
        }
    } else {
        use mlflow::api::error::GetError;
        let experiment = client.get_experiment_by_name(&args.experiment);
        match experiment {
            Ok(experiment) => {
                println!(
                    "Experiment {} with id {} was fetched successfully!",
                    experiment.name,
                    experiment.experiment_id.as_ref(),
                );
                experiment
            }
            Err(GetError::DoesNotExist(name)) => {
                println!("The experiment {} does not exists.", name);
                println!("Run again with the -c or --create flag to create a new experiment.");
                return Ok(());
            }
            Err(err) => {
                println!("Failed to get experiment:\n {}", err);
                return Ok(());
            }
        }
    };

    for i in 0..args.runs {
        println!("Executing run {}", i);
        let mut run = TrackingRun::new();
        run.log_param("i", &format!("{}", i));
        run.log_param("constant", "42");
        let mut rng = WyRand::new_seed(i.into());
        for s in 0..10 {
            let int: f64 = rng.generate::<u16>().into();
            let max: f64 = std::u16::MAX.into();
            let value = int / max;
            run.log_metric("rand", value, s);
        }
        run.submit(&mut client, &experiment.experiment_id)?;
    }

    Ok(())
}
