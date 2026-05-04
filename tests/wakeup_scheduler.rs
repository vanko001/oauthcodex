use oauthcodex::domain::codex_models::*;
use oauthcodex::domain::wakeup::WakeupScheduler;

fn sample_schedule_daily() -> WakeupSchedule {
    WakeupSchedule {
        kind: WakeupScheduleKind::Daily,
        time: Some("09:00".into()),
        interval_minutes: None,
        delay_seconds: None,
        days_of_week: None,
    }
}

fn sample_schedule_weekly() -> WakeupSchedule {
    WakeupSchedule {
        kind: WakeupScheduleKind::Weekly,
        time: Some("10:00".into()),
        interval_minutes: None,
        delay_seconds: None,
        days_of_week: Some(vec![1, 3, 5]),
    }
}

fn sample_schedule_interval() -> WakeupSchedule {
    WakeupSchedule {
        kind: WakeupScheduleKind::Interval,
        time: None,
        interval_minutes: Some(60),
        delay_seconds: None,
        days_of_week: None,
    }
}

#[test]
fn test_create_wakeup_task() {
    let mut scheduler = WakeupScheduler::new();
    let id = scheduler
        .create_task(
            "Daily Refresh".into(),
            sample_schedule_daily(),
            true,
            Some("acct_001".into()),
            Some("preset_refresh".into()),
            Some("Refresh all accounts daily".into()),
        )
        .expect("create task");

    assert!(id.starts_with("wt_"));
    assert_eq!(scheduler.tasks().len(), 1);
    let task = &scheduler.tasks()[0];
    assert_eq!(task.name, "Daily Refresh");
    assert!(task.enabled);
    assert_eq!(task.account_id, Some("acct_001".into()));
    assert_eq!(task.preset_id, Some("preset_refresh".into()));
}

#[test]
fn test_update_wakeup_task() {
    let mut scheduler = WakeupScheduler::new();
    let id = scheduler
        .create_task(
            "Old Name".into(),
            sample_schedule_daily(),
            true,
            None,
            None,
            None,
        )
        .expect("create");

    let updates = WakeupTaskUpdate {
        name: Some("Updated Name".into()),
        schedule: Some(sample_schedule_interval()),
        enabled: Some(false),
        account_id: Some("acct_new".into()),
    };
    scheduler.update_task(&id, updates).expect("update");

    let task = &scheduler.tasks()[0];
    assert_eq!(task.name, "Updated Name");
    assert_eq!(task.schedule.kind, WakeupScheduleKind::Interval);
    assert!(!task.enabled);
    assert_eq!(task.account_id, Some("acct_new".into()));
}

#[test]
fn test_delete_wakeup_task() {
    let mut scheduler = WakeupScheduler::new();
    let id = scheduler
        .create_task(
            "To Delete".into(),
            sample_schedule_daily(),
            true,
            None,
            None,
            None,
        )
        .expect("create");
    assert_eq!(scheduler.tasks().len(), 1);

    scheduler.delete_task(&id).expect("delete");
    assert!(scheduler.tasks().is_empty());
}

#[test]
fn test_run_task_records_history() {
    let mut scheduler = WakeupScheduler::new();
    let id = scheduler
        .create_task(
            "Test Task".into(),
            sample_schedule_daily(),
            true,
            None,
            None,
            None,
        )
        .expect("create");

    scheduler.run_task(&id).expect("run");

    assert_eq!(scheduler.history().len(), 1);
    let entry = &scheduler.history()[0];
    assert_eq!(entry.task_id, id);
    assert_eq!(entry.status, "completed");
    assert!(entry.duration_ms.is_some());
    assert!(entry.error.is_none());
}

#[test]
fn test_cancel_task() {
    let mut scheduler = WakeupScheduler::new();
    let id = scheduler
        .create_task(
            "Cancelable".into(),
            sample_schedule_daily(),
            true,
            None,
            None,
            None,
        )
        .expect("create");
    assert!(scheduler.tasks()[0].enabled);

    scheduler.cancel_task(&id).expect("cancel");
    assert!(!scheduler.tasks()[0].enabled);
}

#[test]
fn test_calculate_next_run_daily() {
    let schedule = sample_schedule_daily();
    let result = WakeupScheduler::calculate_next_run(&schedule);
    assert!(result.is_some());
    assert!(result.unwrap().contains("T09:00:00"));
}

#[test]
fn test_calculate_next_run_weekly() {
    let schedule = sample_schedule_weekly();
    let result = WakeupScheduler::calculate_next_run(&schedule);
    assert!(result.is_some());
    assert!(result.unwrap().contains("T10:00:00"));
}

#[test]
fn test_calculate_next_run_interval() {
    let schedule = sample_schedule_interval();
    let result = WakeupScheduler::calculate_next_run(&schedule);
    assert!(result.is_some());
}

#[test]
fn test_run_enabled_tasks() {
    let mut scheduler = WakeupScheduler::new();
    let id_a = scheduler
        .create_task(
            "Enabled Task".into(),
            sample_schedule_daily(),
            true,
            None,
            None,
            None,
        )
        .expect("create enabled");
    scheduler
        .create_task(
            "Disabled Task".into(),
            sample_schedule_daily(),
            false,
            None,
            None,
            None,
        )
        .expect("create disabled");

    let run_ids = scheduler.run_enabled_tasks();
    assert_eq!(run_ids, vec![id_a]);
    assert_eq!(scheduler.history().len(), 1);
}

#[test]
fn test_load_save_state_persistence() {
    let mut scheduler = WakeupScheduler::new();
    scheduler
        .create_task(
            "Persistable".into(),
            sample_schedule_daily(),
            true,
            None,
            None,
            None,
        )
        .expect("create");

    let state = scheduler.to_state().expect("state");
    assert_eq!(state.tasks.len(), 1);

    let restored = WakeupScheduler::from_state(&state);
    assert_eq!(restored.tasks().len(), 1);
    assert_eq!(restored.tasks()[0].name, "Persistable");
}

#[test]
fn test_load_save_empty_state() {
    let scheduler = WakeupScheduler::new();
    assert!(scheduler.to_state().is_none());
}

#[test]
fn test_history_trim_to_max_1000() {
    let mut scheduler = WakeupScheduler::new();
    let id = scheduler
        .create_task(
            "Bulk Task".into(),
            sample_schedule_daily(),
            true,
            None,
            None,
            None,
        )
        .expect("create");

    for _ in 0..1050 {
        scheduler.run_task(&id).ok();
    }

    assert!(scheduler.history().len() <= 1000);
}
