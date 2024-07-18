use crate::manager::{CurrentState, CurrentStateReason};
use crate::time::Timestamp;

#[derive(Clone, Copy, Debug)]
pub enum StateChangeKind {
    BreakTimerUnlockable,
    BreakTimerLocked,
    RangeLocked(u64),
    RangeUnlocked(u64),
    RequirementLocked(u64),
}

#[derive(Clone, Debug)]
pub struct StateChange {
    pub kind: StateChangeKind,
    pub time: Timestamp,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum SimulatorError {
    LockNotFound(u64),
    DuplicateLock(u64),
}

struct Locks {
    locks: Vec<u64>,
}

impl Locks {
    fn add_lock(&mut self, id: u64) -> Result<(), SimulatorError> {
        if self.locks.iter().any(|&lock_id| lock_id == id) {
            Err(SimulatorError::DuplicateLock(id))
        } else {
            self.locks.push(id);
            Ok(())
        }
    }
    fn unlock(&mut self, id: u64) -> Result<(), SimulatorError> {
        let index = self
            .locks
            .iter()
            .position(|&lock_id| lock_id == id)
            .ok_or(SimulatorError::LockNotFound(id))?;
        self.locks.remove(index);
        Ok(())
    }
    fn is_empty(&self) -> bool {
        self.locks.is_empty()
    }
    fn first(&self) -> Option<u64> {
        self.locks.first().copied()
    }
    fn new() -> Self {
        Self { locks: Vec::new() }
    }
}

#[derive(Debug)]
pub struct SimulatorResult {
    pub target_state: CurrentState,
    pub until: Option<Timestamp>,
    pub reason: CurrentStateReason,
}

pub struct Simulator {
    changes: Vec<StateChange>,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }
    pub fn push(&mut self, change: StateChange) {
        self.changes.push(change);
    }
    pub fn run(&mut self, target_time: Timestamp) -> Result<SimulatorResult, SimulatorError> {
        // stable sort preserves original order of state changes with the same time
        // state changes that were pushed earlier get higher priority when determining the reason
        self.changes.sort_by_key(|sc| sc.time);
        let mut locked_ranges = Locks::new();
        let mut locked_requirements = Locks::new();
        let mut break_timer_state = CurrentState::Unlocked;
        let mut simulator_state = CurrentState::Unlocked;
        let mut simulator_result: Option<SimulatorResult> = None;
        for change in &self.changes {
            use StateChangeKind::*;
            match change.kind {
                BreakTimerUnlockable => break_timer_state = CurrentState::Unlockable,
                BreakTimerLocked => break_timer_state = CurrentState::Locked,
                RangeLocked(id) => locked_ranges.add_lock(id)?,
                RangeUnlocked(id) => locked_ranges.unlock(id)?,
                RequirementLocked(id) => locked_requirements.add_lock(id)?,
            }
            let state_after_change =
                Self::calc_state(&locked_ranges, &locked_requirements, break_timer_state);
            if simulator_state != state_after_change {
                if change.time > target_time {
                    simulator_result = Some(SimulatorResult {
                        target_state: simulator_state,
                        until: Some(change.time),
                        reason: match change.kind {
                            StateChangeKind::BreakTimerUnlockable
                            | StateChangeKind::BreakTimerLocked => CurrentStateReason::BreakTimer,
                            StateChangeKind::RangeLocked(id)
                            | StateChangeKind::RangeUnlocked(id) => {
                                CurrentStateReason::LockedTimeRange { id }
                            }
                            StateChangeKind::RequirementLocked(id) => {
                                CurrentStateReason::RequirementNotMet { id }
                            }
                        },
                    });
                    break;
                } else {
                    simulator_state = state_after_change;
                }
            }
        }
        if let Some(simulator_result) = simulator_result {
            Ok(simulator_result)
        } else {
            Ok(SimulatorResult {
                target_state: simulator_state,
                until: None,
                reason: match simulator_state {
                    CurrentState::Unlocked => CurrentStateReason::NoConstraints,
                    CurrentState::Unlockable => CurrentStateReason::BreakTimer,
                    CurrentState::Locked => {
                        if let Some(id) = locked_requirements.first() {
                            CurrentStateReason::RequirementNotMet { id }
                        } else if let Some(id) = locked_ranges.first() {
                            CurrentStateReason::LockedTimeRange { id }
                        } else {
                            CurrentStateReason::BreakTimer
                        }
                    }
                },
            })
        }
    }
    fn calc_state(
        locked_ranges: &Locks,
        locked_requirements: &Locks,
        break_timer_state: CurrentState,
    ) -> CurrentState {
        if locked_ranges.is_empty() && locked_requirements.is_empty() {
            break_timer_state
        } else {
            CurrentState::Locked
        }
    }
}
