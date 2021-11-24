//! This file contains the trait defining the branch predictor.
//!

use std::{cmp::min, collections::HashMap};

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
        pc ^ self.hist_to_u32()
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
    lhist_bits: u32,
    pc_index: u32,
    g_state: Vec<TwoBitCounterState>,
    l_state: Vec<TwoBitCounterState>,
    m_state: TwoBitCounterState,
    ghist: usize,
}

#[allow(unused_variables)]
impl TournamentPredictor {
    pub fn new(ghist_bits: u32, lhist_bits: u32, pc_index: u32) -> TournamentPredictor {
        TournamentPredictor {
            ghist_bits,
            lhist_bits,
            pc_index,
            g_state: vec![TwoBitCounterState::WeakNot; usize::pow(2, ghist_bits) as usize],
            l_state: vec![TwoBitCounterState::WeakNot; u32::pow(2, lhist_bits) as usize],
            m_state: TwoBitCounterState::WeakNot,
            ghist: 0,
        }
    }

    fn make_local_prediction(&self, pc: u32) -> BranchResult {
        // println!("local");
        let l_index = (pc & ((1 << self.lhist_bits) - 1)) as usize;
        // println!("pc: {:#?}", pc);
        // println!("l_state: {:#?}", self.l_state);

        // println!("pred: {:#?}", self.l_state[l_index].to_branch_result());
        self.l_state[l_index].to_branch_result()
    }

    fn train_local_predictor(&mut self, pc: u32, outcome: BranchResult) {
        // println!("outcome: {:#?}", outcome);
        // println!("outcome: {:#?}", outcome.clone() as usize);
        // println!("pc: 0b{:032b}", pc);
        // println!("pc: {}", (pc & ((1 << self.lhist_bits) -1)));

        let l_index = (pc & ((1 << self.lhist_bits) - 1)) as usize;
        // println!("pc: {:#?}", l_index);
        self.l_state[l_index] = self.l_state[l_index].shift_result(outcome);
        // println!("l_state: {:#?}", self.l_state);
    }

    fn make_global_prediction(&self, pc: u32) -> BranchResult {
        // println!("global");
        let g_index = self.ghist & ((1 << self.ghist_bits) - 1);
        // println!("g_hist: {:#?}", self.ghist);
        // println!("g_state: {:#?}", self.g_state);
        // println!("pred: {:#?}", self.g_state[g_index].to_branch_result());
        self.g_state[g_index].to_branch_result()
    }

    fn train_global_predictor(&mut self, pc: u32, outcome: BranchResult) {
        // println!("outcome: {:#?}", outcome);
        // println!("outcome: {:#?}", outcome.clone() as usize);
        // println!("g_hist: {:#?}", self.ghist);
        self.g_state[self.ghist] = self.g_state[self.ghist].shift_result(outcome.clone());
        // println!("g_state: {:#?}", self.g_state);
        self.ghist = ((self.ghist << 1) & ((1 << self.ghist_bits) - 1)) | outcome as usize;
    }
}

#[allow(unused_variables)]
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

pub struct CustomPredictor;

impl CustomPredictor {
    pub fn new() -> CustomPredictor {
        CustomPredictor {}
    }
}

#[allow(unused_variables)]
impl Predictor for CustomPredictor {
    fn make_prediction(&self, pc: u32) -> BranchResult {
        todo!()
    }

    fn train_predictor(&mut self, pc: u32, outcome: BranchResult) {
        todo!()
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
