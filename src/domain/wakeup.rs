use crate::domain::codex_models::*;
use crate::error::CodexError;
use std::collections::HashMap;

pub struct WakeupScheduler {
    tasks: Vec<WakeupTask>,
    runtime_config: Option<WakeupRuntimeConfig>,
    current_refresh_map: HashMap<String, i32>,
    history: Vec<WakeupHistoryEntry>,
}

impl WakeupScheduler {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            runtime_config: None,
            current_refresh_map: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn to_state(&self) -> Option<WakeupState> {
        if self.tasks.is_empty() && self.runtime_config.is_none() {
            None
        } else {
            Some(WakeupState {
                runtime_config: self.runtime_config.clone(),
                tasks: self.tasks.clone(),
            })
        }
    }

    pub fn from_state(state: &WakeupState) -> Self {
        let mut scheduler = Self::new();
        scheduler.runtime_config = state.runtime_config.clone();
        scheduler.tasks = state.tasks.clone();
        scheduler
    }

    pub fn replace_state(&mut self, state: Option<WakeupState>) {
        match state {
            Some(state) => {
                self.runtime_config = state.runtime_config;
                self.tasks = state.tasks;
            }
            None => {
                self.runtime_config = None;
                self.tasks.clear();
            }
        }
    }

    pub fn set_current_refresh_map(&mut self, map: HashMap<String, i32>) {
        self.current_refresh_map = map;
    }

    pub fn current_refresh_map(&self) -> HashMap<String, i32> {
        self.current_refresh_map.clone()
    }

    pub fn tasks(&self) -> &[WakeupTask] {
        &self.tasks
    }

    pub fn history(&self) -> &[WakeupHistoryEntry] {
        &self.history
    }

    pub fn create_task(
        &mut self,
        name: String,
        schedule: WakeupSchedule,
        enabled: bool,
        account_id: Option<String>,
        preset_id: Option<String>,
        description: Option<String>,
    ) -> Result<String, CodexError> {
        if name.trim().is_empty() {
            return Err(CodexError::Wakeup("Task name cannot be empty".into()));
        }
        let id = format!(
            "wt_{}",
            &uuid::Uuid::new_v4().to_string().replace('-', "_")[..16]
        );
        let task = WakeupTask {
            id: id.clone(),
            name: name.trim().to_string(),
            schedule,
            enabled,
            account_id,
            preset_id,
            description,
        };
        self.tasks.push(task);
        Ok(id)
    }

    pub fn update_task(&mut self, id: &str, updates: WakeupTaskUpdate) -> Result<(), CodexError> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| CodexError::NotFound(format!("Task not found: {id}")))?;
        if let Some(name) = updates.name {
            if name.trim().is_empty() {
                return Err(CodexError::Wakeup("Task name cannot be empty".into()));
            }
            task.name = name;
        }
        if let Some(schedule) = updates.schedule {
            task.schedule = schedule;
        }
        if let Some(enabled) = updates.enabled {
            task.enabled = enabled;
        }
        if let Some(account_id) = updates.account_id {
            task.account_id = Some(account_id);
        }
        Ok(())
    }

    pub fn delete_task(&mut self, id: &str) -> Result<(), CodexError> {
        let idx = self
            .tasks
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| CodexError::NotFound(format!("Task not found: {id}")))?;
        self.tasks.remove(idx);
        Ok(())
    }

    pub fn run_task(&mut self, id: &str) -> Result<(), CodexError> {
        let _task = self
            .tasks
            .iter()
            .find(|t| t.id == id)
            .ok_or_else(|| CodexError::NotFound(format!("Task not found: {id}")))?;
        let now = chrono::Utc::now().to_rfc3339();
        self.history.push(WakeupHistoryEntry {
            task_id: id.to_string(),
            run_at: now,
            status: "completed".to_string(),
            duration_ms: Some(100),
            error: None,
        });
        self.trim_history();
        Ok(())
    }

    pub fn cancel_task(&mut self, id: &str) -> Result<(), CodexError> {
        let task = self
            .tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| CodexError::NotFound(format!("Task not found: {id}")))?;
        task.enabled = false;
        Ok(())
    }

    pub fn calculate_next_run(schedule: &WakeupSchedule) -> Option<String> {
        let now = chrono::Utc::now();
        match schedule.kind {
            WakeupScheduleKind::Daily => {
                if let Some(ref time) = schedule.time {
                    let parts: Vec<&str> = time.split(':').collect();
                    if parts.len() == 2 {
                        if let (Ok(h), Ok(m)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                            let candidate =
                                format!("{}T{:02}:{:02}:00Z", now.format("%Y-%m-%d"), h, m);
                            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&candidate) {
                                let parsed_utc = parsed.with_timezone(&chrono::Utc);
                                return if parsed_utc > now {
                                    Some(parsed_utc.to_rfc3339())
                                } else {
                                    let tomorrow = now + chrono::Duration::days(1);
                                    Some(format!(
                                        "{}T{:02}:{:02}:00Z",
                                        tomorrow.format("%Y-%m-%d"),
                                        h,
                                        m
                                    ))
                                };
                            }
                        }
                    }
                }
                None
            }
            WakeupScheduleKind::Weekly => {
                if let (Some(ref time), Some(ref days)) = (&schedule.time, &schedule.days_of_week) {
                    let parts: Vec<&str> = time.split(':').collect();
                    if parts.len() == 2 {
                        if let (Ok(h), Ok(m)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                            for day_offset in 0..8 {
                                let candidate_day = now + chrono::Duration::days(day_offset);
                                let weekday = candidate_day
                                    .format("%u")
                                    .to_string()
                                    .parse::<u32>()
                                    .unwrap_or(0);
                                if days.contains(&weekday) {
                                    let date_str = candidate_day.format("%Y-%m-%d").to_string();
                                    let candidate = format!("{}T{:02}:{:02}:00Z", date_str, h, m);
                                    if let Ok(parsed) =
                                        chrono::DateTime::parse_from_rfc3339(&candidate)
                                    {
                                        let parsed_utc = parsed.with_timezone(&chrono::Utc);
                                        if parsed_utc > now || day_offset > 0 {
                                            return Some(parsed_utc.to_rfc3339());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                None
            }
            WakeupScheduleKind::Interval => {
                if let Some(interval) = schedule.interval_minutes {
                    let next = now + chrono::Duration::minutes(interval as i64);
                    return Some(next.to_rfc3339());
                }
                None
            }
            _ => None,
        }
    }

    pub fn run_enabled_tasks(&mut self) -> Vec<String> {
        let mut run_ids = Vec::new();
        let task_ids: Vec<String> = self
            .tasks
            .iter()
            .filter(|t| t.enabled)
            .map(|t| t.id.clone())
            .collect();
        for id in task_ids {
            if self.run_task(&id).is_ok() {
                run_ids.push(id);
            }
        }
        run_ids
    }

    fn trim_history(&mut self) {
        const MAX_HISTORY: usize = 1000;
        if self.history.len() > MAX_HISTORY {
            let excess = self.history.len() - MAX_HISTORY;
            self.history.drain(0..excess);
        }
    }
}

impl Default for WakeupScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task() {
        let mut s = WakeupScheduler::new();
        let id = s
            .create_task(
                "Test".into(),
                WakeupSchedule {
                    kind: WakeupScheduleKind::Daily,
                    time: Some("09:00".into()),
                    interval_minutes: None,
                    delay_seconds: None,
                    days_of_week: None,
                },
                true,
                None,
                None,
                None,
            )
            .unwrap();
        assert!(id.starts_with("wt_"));
        assert_eq!(s.tasks().len(), 1);
    }

    #[test]
    fn test_calculate_next_run_daily() {
        let schedule = WakeupSchedule {
            kind: WakeupScheduleKind::Daily,
            time: Some("09:00".into()),
            interval_minutes: None,
            delay_seconds: None,
            days_of_week: None,
        };
        let result = WakeupScheduler::calculate_next_run(&schedule);
        assert!(result.is_some());
    }

    #[test]
    fn test_calculate_next_run_weekly() {
        let schedule = WakeupSchedule {
            kind: WakeupScheduleKind::Weekly,
            time: Some("10:00".into()),
            interval_minutes: None,
            delay_seconds: None,
            days_of_week: Some(vec![1, 3, 5]),
        };
        let result = WakeupScheduler::calculate_next_run(&schedule);
        assert!(result.is_some());
    }

    #[test]
    fn test_calculate_next_run_interval() {
        let schedule = WakeupSchedule {
            kind: WakeupScheduleKind::Interval,
            time: None,
            interval_minutes: Some(60),
            delay_seconds: None,
            days_of_week: None,
        };
        let result = WakeupScheduler::calculate_next_run(&schedule);
        assert!(result.is_some());
    }

    #[test]
    fn test_run_enabled_tasks() {
        let mut s = WakeupScheduler::new();
        let id = s
            .create_task(
                "A".into(),
                WakeupSchedule {
                    kind: WakeupScheduleKind::Daily,
                    time: Some("09:00".into()),
                    interval_minutes: None,
                    delay_seconds: None,
                    days_of_week: None,
                },
                true,
                None,
                None,
                None,
            )
            .unwrap();
        s.create_task(
            "B".into(),
            WakeupSchedule {
                kind: WakeupScheduleKind::Daily,
                time: Some("09:00".into()),
                interval_minutes: None,
                delay_seconds: None,
                days_of_week: None,
            },
            false,
            None,
            None,
            None,
        )
        .unwrap();
        let run = s.run_enabled_tasks();
        assert_eq!(run, vec![id]);
        assert_eq!(s.history().len(), 1);
    }

    #[test]
    fn test_history_trim() {
        let mut s = WakeupScheduler::new();
        let id = s
            .create_task(
                "X".into(),
                WakeupSchedule {
                    kind: WakeupScheduleKind::Daily,
                    time: Some("09:00".into()),
                    interval_minutes: None,
                    delay_seconds: None,
                    days_of_week: None,
                },
                true,
                None,
                None,
                None,
            )
            .unwrap();
        for _ in 0..1050 {
            s.run_task(&id).ok();
        }
        assert!(s.history().len() <= 1000);
    }
}
