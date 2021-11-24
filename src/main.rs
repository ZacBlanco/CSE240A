use std::io::BufRead;
use std::str::FromStr;

use structopt::StructOpt;

use crate::predictor::CustomPredictor;
use crate::predictor::GSharePredictor;
use crate::predictor::StaticPredictor;
use crate::predictor::TournamentPredictor;
mod predictor;

#[derive(PartialEq, Debug, Clone)]
pub enum BranchResult {
    Taken,
    NotTaken,
}
// ask for the order or the thing above
#[derive(Debug)]
enum Predictors {
    STATIC,
    GSHARE(u32),
    TOURNAMENT(u32, u32, u32),
    CUSTOM,
}

impl FromStr for Predictors {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("static") {
            return Ok(Predictors::STATIC);
        } else if s.starts_with("gshare") {
            let s = s.to_string();
            let nums = s.trim_start_matches("gshare").split(":");
            let nums = nums
                .filter(|s| *s != "")
                .map(|v| v.parse::<u32>().unwrap())
                .collect::<Vec<u32>>();
            if nums.len() < 1 {
                return Err(String::from(
                    "gshare:<history bits> argument required for gshare",
                ));
            }
            return Ok(Predictors::GSHARE(nums[0]));
        } else if s.starts_with("tournament") {
            let s = s.to_string();
            let nums = s.trim_start_matches("tournament").split(":");
            let nums = nums
                .filter(|s| *s != "")
                .map(|v| v.parse::<u32>().unwrap())
                .collect::<Vec<u32>>();
            if nums.len() < 3 {
                return Err(String::from(
                    "tournament:<ghistory>:<lhistory>:<index> bits required for tournament",
                ));
            }
            return Ok(Predictors::TOURNAMENT(nums[0], nums[1], nums[2]));
        } else if s.starts_with("custom") {
            return Ok(Predictors::CUSTOM);
        } else {
            return Err(String::from("invalid predictor type"));
        }
    }

    type Err = String;
}

#[derive(StructOpt, Debug)]
#[structopt(name = "predictor")]
struct Opt {
    #[structopt(long)]
    predictor: Predictors,

    #[structopt(long)]
    verbose: bool,
}

fn main() -> Result<(), std::io::Error> {
    let args = Opt::from_args();
    let mut predictor: Box<dyn predictor::Predictor> = match args.predictor {
        Predictors::STATIC => Box::new(StaticPredictor {}),
        Predictors::GSHARE(hist_bits) => Box::new(GSharePredictor::new(hist_bits)),
        Predictors::TOURNAMENT(ghist, lhist, pc_index) => {
            Box::new(TournamentPredictor::new(ghist, lhist, pc_index))
        }
        Predictors::CUSTOM => Box::new(CustomPredictor::new()),
    };

    let mut num_branches: u32 = 0;
    let mut mispredictions: u32 = 0;

    let mut buf = String::new();
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    loop {
        buf.clear();
        let bytes_read = stdin.read_line(&mut buf);
        match bytes_read {
            Ok(0) => break,
            Ok(_) => {
                let mut parts = buf.split(" ");
                let raw_pc = parts.next().unwrap();
                let pc = u32::from_str_radix(&raw_pc[2..], 16).unwrap();
                let raw_outcome = parts.next().unwrap();
                let outcome = match &raw_outcome
                    .chars()
                    .nth(0)
                    .unwrap()
                    .to_string()
                    .parse::<u8>()
                    .unwrap()
                {
                    0 => BranchResult::NotTaken,
                    1 => BranchResult::Taken,
                    _ => panic!("invalid branch outcome"),
                };
                num_branches += 1;
                let prediction = predictor.make_prediction(pc);
                if prediction != outcome {
                    mispredictions += 1;
                }
                if args.verbose {
                    println!("{:?}", prediction);
                }
                predictor.train_predictor(pc, outcome);
            }
            Err(e) => panic!("Failed to read stdin: {:?}", e),
        }
    }

    println!("branches:\t\t{}", num_branches);
    println!("incorrect:\t\t{}", mispredictions);
    println!(
        "misprediction rate:\t{:.2}%",
        100.0 * (mispredictions as f64 / num_branches as f64)
    );
    Ok(())
}
