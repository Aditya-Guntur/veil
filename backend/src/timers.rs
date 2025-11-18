use crate::{STATE, types::RoundState};
use ic_cdk_timers::{set_timer, set_timer_interval, TimerId};
use std::time::Duration;
use std::cell::RefCell;

thread_local! {
    static ROUND_TIMER: RefCell<Option<TimerId>> = RefCell::new(None);
}

/// Start the automatic round progression timer
pub fn start_round_timer() {
    ic_cdk::println!("Starting automatic round timer");
    
    // Check every 5 seconds if round should progress
    let timer_id = set_timer_interval(Duration::from_secs(5), || {
        ic_cdk::spawn(check_and_progress_round());
    });
    
    ROUND_TIMER.with(|timer| {
        *timer.borrow_mut() = Some(timer_id);
    });
    
    ic_cdk::println!("Round timer started");
}

/// Check if round should progress and do so if needed
async fn check_and_progress_round() {
    let (should_progress, current_state, round_id) = STATE.with(|s| {
        let state = s.borrow();
        let current_time = ic_cdk::api::time();
        let elapsed = current_time.saturating_sub(state.round_start_time);
        
        let should_progress = state.round_state == RoundState::Active
            && elapsed >= state.round_duration_ns;
        
        (should_progress, state.round_state.clone(), state.round_id)
    });
    
    if should_progress {
        ic_cdk::println!(
            "Round {} time expired. Auto-progressing to clearing...",
            round_id
        );
        
        // Trigger clearing
        let result = crate::admin_run_clearing().await;
        ic_cdk::println!("Auto-clearing result: {}", result);
        
        // After completion, wait 10 seconds then start new round
        set_timer(Duration::from_secs(10), || {
            ic_cdk::spawn(auto_start_next_round());
        });
    }
    
    // Auto-start if in Completed state for too long
    if current_state == RoundState::Completed {
        let time_since_completion = STATE.with(|s| {
            let state = s.borrow();
            ic_cdk::api::time().saturating_sub(state.round_start_time + state.round_duration_ns)
        });
        
        // If 15 seconds have passed since completion, start new round
        if time_since_completion >= 15_000_000_000 {
            ic_cdk::println!("Auto-starting next round after completion delay");
            ic_cdk::spawn(auto_start_next_round());
        }
    }
}

/// Automatically start the next round
async fn auto_start_next_round() {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        
        if state.round_state == RoundState::Completed || state.round_state == RoundState::Pending {
            state.round_id += 1;
            state.round_state = RoundState::Active;
            state.round_start_time = ic_cdk::api::time();
            
            ic_cdk::println!(
                "Auto-started round {}. Duration: {}s",
                state.round_id,
                state.round_duration_ns / 1_000_000_000
            );
        }
    });
}

/// Stop the automatic round timer (for testing/admin)
#[ic_cdk_macros::update]
pub fn stop_round_timer() -> String {
    ROUND_TIMER.with(|timer| {
        if let Some(timer_id) = timer.borrow_mut().take() {
            ic_cdk_timers::clear_timer(timer_id);
            "Round timer stopped".to_string()
        } else {
            "No active timer to stop".to_string()
        }
    })
}

/// Manually trigger round progression (for testing)
#[ic_cdk_macros::update]
pub async fn force_progress_round() -> String {
    check_and_progress_round().await;
    "Round progression triggered".to_string()
}

/// Set custom round duration (for testing)
#[ic_cdk_macros::update]
pub fn set_round_duration(seconds: u64) -> String {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.round_duration_ns = seconds * 1_000_000_000;
        format!("Round duration set to {} seconds", seconds)
    })
}