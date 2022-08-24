use crate::config::{LockedTimeRangeConfig, RequirementConfig};
use crate::server::{ClientHandlingError, Response};
use crate::simulator::{Simulator, StateChange, StateChangeKind};
use crate::time::{Duration, HourMinute, LocalDate, Timestamp};
use serde::{Deserialize, Serialize};
use std::process::{Child, Command};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Requirement {
    id: u64,
    name: String,
    due: Timestamp,
    complete: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TimeRange {
    id: u64,
    start: Option<Timestamp>,
    end: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum BreakTimer {
    Unlocked { until: Timestamp },
    Locked { until: Timestamp },
    Unlockable,
}

struct BreakTimerManager {
    timer: BreakTimer,
    work_period_duration: Duration,
    break_duration: Duration,
}

impl BreakTimerManager {
    fn new(work_period_duration: Duration, break_duration: Duration) -> Self {
        Self {
            timer: BreakTimer::Unlockable,
            work_period_duration,
            break_duration,
        }
    }
    fn unlock(&mut self, current_time: Timestamp) -> Result<(), String> {
        self.refresh(current_time);
        match self.timer {
            BreakTimer::Unlockable => {
                self.timer = BreakTimer::Unlocked {
                    until: current_time + self.work_period_duration,
                };
                Ok(())
            }
            BreakTimer::Locked { until: _ } => Err("Break timer is locked.".to_owned()),
            BreakTimer::Unlocked { until: _ } => Err("Break timer is already unlocked.".to_owned()),
        }
    }
    fn lock(&mut self, current_time: Timestamp) -> Result<(), String> {
        self.refresh(current_time);
        match self.timer {
            BreakTimer::Unlocked { until: _ } => {
                self.timer = BreakTimer::Locked {
                    until: current_time + self.break_duration,
                };
                Ok(())
            }
            _ => Err("Break timer is not unlocked.".to_owned()),
        }
    }
    fn refresh(&mut self, current_time: Timestamp) {
        if let BreakTimer::Unlocked { until } = self.timer {
            if current_time >= until {
                self.timer = BreakTimer::Locked {
                    until: until + self.break_duration,
                };
            }
        }
        if let BreakTimer::Locked { until } = self.timer {
            if current_time >= until {
                self.timer = BreakTimer::Unlockable;
            }
        }
    }
    fn lock_if_unlocked(&mut self, current_time: Timestamp) {
        if let Ok(_) = self.lock(current_time) {}
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentState {
    Unlocked,
    Locked,
    Unlockable,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum CurrentStateReason {
    BreakTimer,
    RequirementNotMet { id: u64 },
    LockedTimeRange { id: u64 },
    NoConstraints,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurrentInfo {
    state: CurrentState,
    until: Option<Timestamp>,
    reason: CurrentStateReason,
    locked_time_ranges: Vec<TimeRange>,
    requirements: Vec<Requirement>,
}
struct Constraints {
    break_timer: BreakTimerManager,
    requirements: Vec<Requirement>,
    locked_time_ranges: Vec<TimeRange>,
}

impl Constraints {
    fn get_current_info(
        &mut self,
        current_time: Timestamp,
    ) -> Result<CurrentInfo, ClientHandlingError> {
        self.break_timer.refresh(current_time);
        let mut simulator = Simulator::new();
        // now we push the state changes into the simulator in the following order:
        // 1. requirements
        // 2. locked time ranges
        // 3. break timer
        // this ensures that if multiple state changes occur at the same time,
        // requirements and locked time ranges will get first and second priority,
        // respectively, when determining the reason
        for requirement in &self.requirements {
            if !requirement.complete {
                simulator.push(StateChange {
                    kind: StateChangeKind::RequirementLocked(requirement.id),
                    time: requirement.due,
                })
            }
        }
        for ltr in &self.locked_time_ranges {
            simulator.push(StateChange {
                kind: StateChangeKind::RangeLocked(ltr.id),
                time: ltr.start.unwrap_or(Timestamp::ZERO),
            });
            if let Some(ltr_end) = ltr.end {
                simulator.push(StateChange {
                    kind: StateChangeKind::RangeUnlocked(ltr.id),
                    time: ltr_end,
                })
            }
        }
        match &self.break_timer.timer {
            BreakTimer::Unlocked { until } => simulator.push(StateChange {
                kind: StateChangeKind::BreakTimerLocked,
                time: *until,
            }),
            BreakTimer::Locked { until } => {
                simulator.push(StateChange {
                    kind: StateChangeKind::BreakTimerLocked,
                    time: Timestamp::ZERO,
                });
                simulator.push(StateChange {
                    kind: StateChangeKind::BreakTimerUnlockable,
                    time: *until,
                });
            }
            BreakTimer::Unlockable => simulator.push(StateChange {
                kind: StateChangeKind::BreakTimerUnlockable,
                time: Timestamp::ZERO,
            }),
        }
        let result = simulator
            .run(current_time)
            .map_err(|e| ClientHandlingError::SimulatorError(e))?;
        Ok(CurrentInfo {
            state: result.target_state,
            until: result.until,
            reason: match result.reason {
                Some(StateChangeKind::BreakTimerUnlockable | StateChangeKind::BreakTimerLocked) => {
                    CurrentStateReason::BreakTimer
                }
                Some(StateChangeKind::RangeLocked(id) | StateChangeKind::RangeUnlocked(id)) => {
                    CurrentStateReason::LockedTimeRange { id }
                }
                Some(StateChangeKind::RequirementLocked(id)) => {
                    CurrentStateReason::RequirementNotMet { id }
                }
                None => CurrentStateReason::NoConstraints,
            },
            locked_time_ranges: self.locked_time_ranges.clone(),
            requirements: self.requirements.clone(),
        })
    }
    fn complete_requirement(&mut self, id: u64) -> Result<(), String> {
        for req in &mut self.requirements {
            if req.id == id {
                if !req.complete {
                    req.complete = true;
                    return Ok(());
                } else {
                    return Err(format!("Requirement {} has already been completed.", id));
                }
            }
        }
        Err(format!("Requirement {} not found.", id))
    }
}

pub struct DiagonatorManager {
    config: DiagonatorManagerConfig,
    diagonator_process: Option<Child>,
    constraints: Constraints,
    current_date: LocalDate,
    id_generator: IdGenerator,
}

impl DiagonatorManager {
    pub fn new(config: DiagonatorManagerConfig) -> Self {
        let break_timer =
            BreakTimerManager::new(config.work_period_duration, config.break_duration);
        Self {
            config,
            diagonator_process: None,
            constraints: Constraints {
                break_timer,
                requirements: Vec::new(),
                locked_time_ranges: Vec::new(),
            },
            current_date: Timestamp::ZERO.get_date(),
            id_generator: IdGenerator::new(),
        }
    }
    pub fn unlock_timer(
        &mut self,
        current_time: Timestamp,
    ) -> Result<Response, ClientHandlingError> {
        let info = self.refresh(current_time)?;
        if matches!(info.state, CurrentState::Unlockable) {
            match self.constraints.break_timer.unlock(current_time) {
                Ok(()) => {
                    self.refresh(current_time)?;
                    Ok(Response::Success)
                }
                Err(msg) => Ok(Response::Error { msg }),
            }
        } else {
            Ok(Response::Error {
                msg: "Session is not unlockable.".to_owned(),
            })
        }
    }
    pub fn lock_timer(&mut self, current_time: Timestamp) -> Result<Response, ClientHandlingError> {
        self.check_running();
        self.refresh(current_time)?;
        match self.constraints.break_timer.lock(current_time) {
            Ok(()) => {
                self.refresh(current_time)?;
                Ok(Response::Success)
            }
            Err(msg) => Ok(Response::Error { msg }),
        }
    }
    pub fn get_info(&mut self, current_time: Timestamp) -> Result<Response, ClientHandlingError> {
        Ok(Response::Info {
            info: self.refresh(current_time)?,
        })
    }
    pub fn complete_requirement(
        &mut self,
        current_time: Timestamp,
        requirement_id: u64,
    ) -> Result<Response, ClientHandlingError> {
        self.refresh(current_time)?;
        match self.constraints.complete_requirement(requirement_id) {
            Ok(()) => {
                self.refresh(current_time)?;
                Ok(Response::Success)
            }
            Err(msg) => Ok(Response::Error { msg }),
        }
    }
    pub fn add_requirement(
        &mut self,
        current_time: Timestamp,
        name: String,
        due: HourMinute,
    ) -> Result<Response, ClientHandlingError> {
        self.refresh(current_time)?;
        self.constraints.requirements.push(Requirement {
            id: self.id_generator.next_id(),
            name,
            due: Timestamp::from_date_hm(&self.current_date, &due),
            complete: false,
        });
        self.refresh(current_time)?;
        Ok(Response::Success)
    }
    fn new_day(&mut self) {
        self.constraints.requirements = self
            .config
            .requirements
            .iter()
            .map(|req| Requirement {
                id: self.id_generator.next_id(),
                name: req.name.clone(),
                due: Timestamp::from_date_hm(&self.current_date, &req.due),
                complete: false,
            })
            .collect();
        self.constraints.locked_time_ranges = self
            .config
            .locked_time_ranges
            .iter()
            .map(|ltr| TimeRange {
                id: self.id_generator.next_id(),
                start: Timestamp::from_date_hm_opt(&self.current_date, &ltr.start),
                end: Timestamp::from_date_hm_opt(&self.current_date, &ltr.end),
            })
            .collect();
    }
    fn check_running(&mut self) {
        if let Some(process) = &mut self.diagonator_process {
            // check if diagonator process has terminated unexpectedly
            if let Ok(Some(_)) = process.try_wait() {
                self.diagonator_process = None;
            }
        }
    }
    fn refresh(&mut self, current_time: Timestamp) -> Result<CurrentInfo, ClientHandlingError> {
        let current_date = current_time.get_date();
        if current_date != self.current_date {
            self.current_date = current_date;
            self.new_day();
        }
        let current_info = self.constraints.get_current_info(current_time)?;

        let diagonator_should_be_running = !(matches!(current_info.state, CurrentState::Unlocked));
        if diagonator_should_be_running {
            self.constraints.break_timer.lock_if_unlocked(current_time);
        }
        match &mut self.diagonator_process {
            Some(process) => {
                if !diagonator_should_be_running {
                    process.kill().expect("Failed to kill diagonator.");
                    process.wait().expect("Failed to wait for diagonator.");
                    self.diagonator_process = None;
                }
            }
            None => {
                if diagonator_should_be_running {
                    self.diagonator_process = Some(
                        Command::new(&self.config.diagonator_command.0)
                            .args(&self.config.diagonator_command.1)
                            .spawn()
                            .expect("Failed to spawn diagonator."),
                    )
                }
            }
        }
        Ok(current_info)
    }
}

pub struct DiagonatorManagerConfig {
    pub diagonator_command: (String, Vec<String>),
    pub requirements: Vec<RequirementConfig>,
    pub locked_time_ranges: Vec<LockedTimeRangeConfig>,
    pub work_period_duration: Duration,
    pub break_duration: Duration,
}

struct IdGenerator {
    last_id: u64,
}

impl IdGenerator {
    fn next_id(&mut self) -> u64 {
        self.last_id += 1;
        self.last_id
    }
    fn new() -> Self {
        Self { last_id: 0 }
    }
}
