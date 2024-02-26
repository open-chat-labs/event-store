use crate::state;

pub fn caller_can_push_events() -> Result<(), String> {
    if state::read(|s| s.can_caller_push_events()) {
        Ok(())
    } else {
        Err(err_message("push"))
    }
}

pub fn caller_can_read_events() -> Result<(), String> {
    if state::read(|s| s.can_caller_read_events()) {
        Ok(())
    } else {
        Err(err_message("read"))
    }
}

fn err_message(action: &'static str) -> String {
    format!("Caller is not authorized to {action} events")
}
