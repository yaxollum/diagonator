use crate::{manager::CurrentState, time::Timestamp};

#[derive(Clone, Copy)]
pub enum StateChangeKind {
    BreakTimerUnlockable,
    BreakTimerLocked,
    RangeLocked(u64),
    RangeUnlocked(u64),
    RequirementLocked(u64),
}

#[derive(Clone)]
pub struct StateChange {
    pub kind: StateChangeKind,
    pub time: Timestamp,
}

#[derive(Debug)]
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
    fn new() -> Self {
        Self { locks: Vec::new() }
    }
}

struct SimulatorState {
    state: CurrentState,
    last_change: Option<StateChange>,
}

pub struct SimulatorResult {
    pub target_state: CurrentState,
    pub until: Option<Timestamp>,
    pub reason: Option<StateChangeKind>,
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
        let mut simulator_state = SimulatorState {
            state: CurrentState::Unlocked,
            last_change: None,
        };
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
            if state_after_change != simulator_state.state {
                if change.time > target_time {
                    simulator_result = Some(SimulatorResult {
                        target_state: simulator_state.state,
                        until: Some(change.time),
                        reason: Some(change.kind),
                    });
                    break;
                } else {
                    simulator_state.state = state_after_change;
                    simulator_state.last_change = Some(change.clone());
                }
            }
        }
        if let Some(simulator_result) = simulator_result {
            Ok(simulator_result)
        } else {
            Ok(SimulatorResult {
                target_state: simulator_state.state,
                until: None,
                reason: simulator_state.last_change.map(|change| change.kind),
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
