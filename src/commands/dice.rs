use rand::Rng;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

#[derive(Debug)]
pub struct DiceRoll {
    pub roll_spec: DiceSpecifier,
    pub modifiers: Vec<Box<DiceModifier>>,
}

#[derive(Debug)]
pub struct DiceSpecifier {
    pub num: u32,
    pub sides: u32,
}

pub struct RollRequest {
    pub dice: Box<Dice>,
    pub roll_values: BTreeMap<(u32, u32), i32>,
    pub spec: DiceSpecifier,
    pub iterations: u32,
    pub cancellations: BTreeSet<(u32, u32)>,
    pub total_modifiers: BTreeMap<u32, i32>,
}

pub struct Roll {
    pub results: Vec<Vec<(i32, bool)>>,
}

impl DiceRoll {
    pub fn roll() -> Result<Roll, String> {
        unimplemented!()
    }
}

impl DiceSpecifier {
    pub fn roll_one(&self, dice: &mut Dice) -> i32 {
        dice.roll(self.sides)
    }

    pub fn roll_all(&self, dice: &mut Dice) -> Vec<i32> {
        (0..self.num).map(|_| dice.roll(self.sides)).collect()
    }
}

pub trait DiceModifier: Debug {
    fn pre_roll(&self, state: &mut RollRequest) -> Result<(), String> {
        let _ = state;
        Ok(())
    }

    fn post_roll(&self, state: &mut RollRequest) -> Result<(), String> {
        let _ = state;
        Ok(())
    }
}

pub trait Dice {
    fn roll(&mut self, sides: u32) -> i32;
}

impl<R> Dice for R
where
    R: Rng,
{
    fn roll(&mut self, sides: u32) -> i32 {
        self.gen_range(0, sides) as i32 + 1
    }
}
