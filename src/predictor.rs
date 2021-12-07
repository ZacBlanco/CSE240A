//! This file contains the trait defining the branch predictor.
//!

use std::{
    cmp::min,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use crate::BranchResult;

#[derive(Clone, Copy, Debug)]
enum TwoBitCounterState {
    StrongNot,
    Strong,
    WeakNot,
    Weak,
}

impl TwoBitCounterState {
    fn shift_result(&self, branch: BranchResult) -> TwoBitCounterState {
        match branch {
            BranchResult::Taken => match &self {
                Self::StrongNot => Self::WeakNot,
                Self::WeakNot => Self::Weak,
                Self::Weak => Self::Strong,
                Self::Strong => Self::Strong,
            },
            BranchResult::NotTaken => match &self {
                Self::StrongNot => Self::StrongNot,
                Self::WeakNot => Self::StrongNot,
                Self::Weak => Self::WeakNot,
                Self::Strong => Self::Weak,
            },
        }
    }

    fn to_branch_result(self) -> BranchResult {
        match self {
            Self::Weak => BranchResult::Taken,
            Self::Strong => BranchResult::Taken,
            Self::WeakNot => BranchResult::NotTaken,
            Self::StrongNot => BranchResult::NotTaken,
        }
    }
}
/// A branch predictor
pub trait Predictor {
    /// Make a prediction for conditional branch instruction at PC 'pc'
    /// Returning TAKEN indicates a prediction of taken; returning NOTTAKEN
    /// indicates a prediction of not taken
    fn make_prediction(&self, pc: u32) -> BranchResult;

    /// Train the predictor the last executed branch at PC 'pc' and with
    /// outcome 'outcome' (true indicates that the branch was taken, false
    /// indicates that the branch was not taken)
    fn train_predictor(&mut self, pc: u32, outcome: BranchResult);
}

pub struct StaticPredictor;

impl Predictor for StaticPredictor {
    fn make_prediction(&self, _pc: u32) -> BranchResult {
        BranchResult::Taken
    }

    fn train_predictor(&mut self, _pc: u32, _outcome: BranchResult) {
        // intentionally empty
    }
}

pub struct GSharePredictor {
    hist_bits: u32,
    history_register: Vec<BranchResult>,
    state_table: HashMap<u32, TwoBitCounterState>,
}

impl GSharePredictor {
    pub fn new(hist_bits: u32) -> GSharePredictor {
        GSharePredictor {
            hist_bits,
            history_register: vec![BranchResult::NotTaken; hist_bits as usize],
            state_table: HashMap::new(),
        }
    }

    fn hist_to_u32(&self) -> u32 {
        let mut x: u32 = 0;
        for i in 0..(min(self.hist_bits, 32)) {
            x <<= 1;
            // println!("0b{:032b}", x);
            // set the bit
            x |= match self.history_register[i as usize] {
                BranchResult::Taken => 0,
                BranchResult::NotTaken => 1,
            };
        }
        x
    }

    fn xor_pc_history(&self, pc: u32) -> u32 {
        // the table can only be indexed up to 2^(m) bits
        // this left shifts a 1 n places to get 0000...1000...000
        // subtract 1 to get a mask of 000001111..111 with 1s in the N
        // least significant places
        let mask = (1  << self.hist_bits) - 1;
        // do the xor first, then apply the mask
        (pc ^ self.hist_to_u32()) & mask
    }
}

impl Predictor for GSharePredictor {
    fn make_prediction(&self, pc: u32) -> BranchResult {
        let table_index = self.xor_pc_history(pc);
        match self.state_table.get(&table_index) {
            Some(state) => state.to_branch_result(),
            None => BranchResult::NotTaken,
        }
    }

    fn train_predictor(&mut self, pc: u32, outcome: BranchResult) {
        let index = self.xor_pc_history(pc);
        self.history_register.rotate_right(1);
        self.history_register[0] = outcome.clone();
        let state = self
            .state_table
            .entry(index)
            .or_insert(TwoBitCounterState::StrongNot);
        *state = state.shift_result(outcome);
    }
}

pub struct TournamentPredictor {
    ghist_bits: u32,
    pc_index: u32,
    g_state: Vec<TwoBitCounterState>,
    l_state: Vec<TwoBitCounterState>,
    l_pattern: Vec<u32>,
    m_state: TwoBitCounterState,
    ghist: usize,
}

impl TournamentPredictor {
    pub fn new(ghist_bits: u32, lhist_bits: u32, pc_index: u32) -> TournamentPredictor {
        TournamentPredictor {
            ghist_bits,
            pc_index,
            g_state: vec![TwoBitCounterState::WeakNot; usize::pow(2, ghist_bits) as usize],
            l_state: vec![TwoBitCounterState::WeakNot; u32::pow(2, lhist_bits) as usize],
            l_pattern: vec![0; u32::pow(2, pc_index) as usize],
            m_state: TwoBitCounterState::WeakNot,
            ghist: 0,
        }
    }

    fn make_local_prediction(&self, pc: u32) -> BranchResult {
        // println!("local");
        let l_pattern_index = (pc & ((1 << self.pc_index) - 1)) as usize;
        let l_index = self.l_pattern[l_pattern_index];
        // println!("pc: {:#?}", pc);
        // println!("l_state: {:#?}", self.l_state);

        // println!("pred: {:#?}", self.l_state[l_index as usize].to_branch_result());
        self.l_state[l_index as usize].to_branch_result()
    }

    fn train_local_predictor(&mut self, pc: u32, outcome: BranchResult) {
        // println!("outcome: {:#?}", outcome);
        // println!("outcome: {:#?}", outcome.clone() as usize);
        // println!("pc: 0b{:032b}", pc);
        // println!("pc: {}", (pc & ((1 << self.pc_index) -1)));
        // println!("l_pattern: {:#?}", self.l_pattern);
        let l_pattern_index = (pc & ((1 << self.pc_index) - 1)) as usize;
        let l_index = self.l_pattern[l_pattern_index];

        // println!("l_index: {:#?}", l_index);
        // println!("l_state: {:#?}", self.l_state);
        self.l_state[l_index as usize] =
            self.l_state[l_index as usize].shift_result(outcome.clone());
        self.l_pattern[l_pattern_index] =
            ((self.l_pattern[l_pattern_index] << 1) & ((1 << self.pc_index) - 1)) | outcome as u32;

        // println!("l_state: {:#?}", self.l_state);
    }

    fn make_global_prediction(&self, _pc: u32) -> BranchResult {
        // println!("global");
        let g_index = self.ghist & ((1 << self.ghist_bits) - 1);
        // println!("g_hist: {:#?}", self.ghist);
        // println!("g_state: {:#?}", self.g_state);
        // println!("pred: {:#?}", self.g_state[g_index].to_branch_result());
        self.g_state[g_index].to_branch_result()
    }

    fn train_global_predictor(&mut self, _pc: u32, outcome: BranchResult) {
        // println!("outcome: {:#?}", outcome);
        // println!("outcome: {:#?}", outcome.clone() as usize);
        // println!("g_hist: {:#?}", self.ghist);
        self.g_state[self.ghist] = self.g_state[self.ghist].shift_result(outcome.clone());
        // println!("g_state: {:#?}", self.g_state);
        self.ghist = ((self.ghist << 1) & ((1 << self.ghist_bits) - 1)) | outcome as usize;
    }
}

impl Predictor for TournamentPredictor {
    fn make_prediction(&self, pc: u32) -> BranchResult {
        match self.m_state {
            TwoBitCounterState::Weak => self.make_local_prediction(pc),
            TwoBitCounterState::Strong => self.make_local_prediction(pc),
            TwoBitCounterState::StrongNot => self.make_global_prediction(pc),
            TwoBitCounterState::WeakNot => self.make_global_prediction(pc),
        }
    }

    fn train_predictor(&mut self, pc: u32, outcome: BranchResult) {
        let local_prediction = self.make_local_prediction(pc);
        let global_prediction = self.make_global_prediction(pc);

        if local_prediction != global_prediction {
            if local_prediction == outcome {
                // println!("switch local");
                self.m_state = self.m_state.shift_result(BranchResult::Taken);
            } else {
                // println!("switch global");
                self.m_state = self.m_state.shift_result(BranchResult::NotTaken);
            }
        }

        self.train_global_predictor(pc, outcome.clone());
        self.train_local_predictor(pc, outcome);
    }
}

/// [CustomPredictor] is a perceptron-based branch predictor based on
/// https://www.cs.utexas.edu/~lin/papers/hpca01.pdf
pub struct CustomPredictor {
    /// size of the global history table, also equivalent to the size of the perceptron
    history_register: Vec<BranchResult>,
    /// a vec representing a table of all perceptrons in the predictor
    perceptrons: Vec<Perceptron>,
    /// a parameter used to determine when training should (or should not) occur.
    theta: u32,
}

type Perceptron = Vec<i32>;

impl CustomPredictor {
    pub fn new(history_length: u32, perceptron_table_size: u32, theta: u32) -> CustomPredictor {
        CustomPredictor {
            history_register: vec![BranchResult::NotTaken; history_length as usize],
            perceptrons: vec![vec![0; history_length as usize]; perceptron_table_size as usize],
            theta,
        }
    }

    /// takes in a pc address and hashes it to an index in the perceptrons table
    fn hash_pc(&self, pc: u32) -> usize {
        let mut hasher = DefaultHasher::new();
        pc.hash(&mut hasher);
        hasher.finish() as usize % self.perceptrons.len()
    }

    /// returns the dot product of a perceptron with the global history table
    fn history_dot(&self, perceptron: &[i32]) -> i32 {
        let mut running_total = 0;
        // iterate over each perceptron weight
        for (idx, weight) in perceptron.iter().enumerate() {
            // the matching result from the global history table
            // if the branch is taken, then it results in a positive score
            // not taken is a negative score
            let x = &self.history_register[idx];
            running_total += match x {
                BranchResult::Taken => *weight,
                BranchResult::NotTaken => -1 * weight,
            }
        }
        running_total
    }

    /// returns the prediction value based on the perceptron dot product
    fn predict(&self, pc: u32) -> i32 {
        let perceptron = &self.perceptrons[self.hash_pc(pc)];
        self.history_dot(perceptron)
    }
}

impl Predictor for CustomPredictor {
    /// makes the prediction based on the branch prediction value.
    /// <= 0 means we predict not to take the branch
    /// > 0 means to take the branch
    fn make_prediction(&self, pc: u32) -> BranchResult {
        match &self.predict(pc).cmp(&0) {
            std::cmp::Ordering::Less => BranchResult::NotTaken,
            std::cmp::Ordering::Equal => BranchResult::NotTaken,
            std::cmp::Ordering::Greater => BranchResult::Taken,
        }
    }

    /// train perceptrons based on the true branch outcome
    /// first, get the original prediction value
    /// if the original prediction was correct AND greater than our theta there is no need to train
    ///
    /// however, if the prediction is wrong and we're under theta, then we need to train the
    /// perceptron
    fn train_predictor(&mut self, pc: u32, outcome: BranchResult) {
        let pred = self.predict(pc);
        if pred.signum() != outcome.to_int().signum() || (pred.abs() as u32) < self.theta {
            // train perceptron, lookup the exact perceptron vec, then perform the training routine
            let percep_idx = self.hash_pc(pc);
            let perceptron = &mut self.perceptrons[percep_idx];
            let t = outcome.to_int();
            // training routine
            for (idx, w) in perceptron.iter_mut().enumerate() {
                let x = self.history_register[idx].to_int();
                *w += t * x;
            }
        }
        // finally, update the history register
        self.history_register.rotate_right(1);
        self.history_register[0] = outcome;
    }
}

#[cfg(test)]
mod test {
    use crate::{predictor::Predictor, BranchResult};

    use super::StaticPredictor;

    #[test]
    fn test_static() {
        let predictor = StaticPredictor {};
        assert_eq!(BranchResult::Taken, predictor.make_prediction(0));
    }
}
