//! This file contains the trait defining the branch predictor.
//!

use crate::BranchResult;

/// A branch predictor
pub trait Predictor {
    /// Make a prediction for conditional branch instruction at PC 'pc'
    /// Returning TAKEN indicates a prediction of taken; returning NOTTAKEN
    /// indicates a prediction of not taken
    fn make_prediction(&self, pc: u32) -> BranchResult;

    /// Train the predictor the last executed branch at PC 'pc' and with
    /// outcome 'outcome' (true indicates that the branch was taken, false
    /// indicates that the branch was not taken)
    fn train_predictor(&self, pc: u32, outcome: BranchResult);
}

pub struct GSharePredictor {
    hist_bits: u32,
}

impl Predictor for GSharePredictor {
    fn make_prediction(&self, pc: u32) -> BranchResult {
        todo!()
    }

    fn train_predictor(&self, pc: u32, outcome: BranchResult) {
        todo!()
    }
}

pub struct StaticPredictor;

impl Predictor for StaticPredictor {
    fn make_prediction(&self, _pc: u32) -> BranchResult {
        return BranchResult::Taken;
    }

    fn train_predictor(&self, _pc: u32, _outcome: BranchResult) {
        // intentionally empty
    }
}


#[cfg(test)]
mod test {
    use crate::{BranchResult, predictor::Predictor};

    use super::StaticPredictor;


  #[test]
  fn test_static() {
    let predictor = StaticPredictor{};
    assert_eq!(BranchResult::Taken, predictor.make_prediction(0));
  }


}
