use chrono::{DateTime, Local, Duration};
use crate::todo::{Todo, Status};

#[derive(Debug)]
pub struct Reminder {
    pub message: String,
    pub emoji: String,
    pub priority: ReminderPriority,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ReminderPriority {
    Info,
    Warning,
    Critical,
}

pub struct ReminderService;

impl ReminderService {
    pub fn new() -> Self {
        Self
    }
    
    pub fn get_reminders(&self, todos: &[Todo]) -> Vec<Reminder> {
        let mut reminders = Vec::new();
        let now = Local::now();
        
        for todo in todos.iter().filter(|t| t.status == Status::Pending) {
            if let Some(due_date) = todo.due_date {
                let due_datetime = DateTime::<Local>::from_naive_utc_and_offset(due_date, *now.offset());
                let time_diff = due_datetime - now;
                
                // Overdue tasks
                if time_diff < Duration::zero() {
                    let days_overdue = (-time_diff).num_days();
                    let hours_overdue = (-time_diff).num_hours();
                    
                    let message = if days_overdue > 0 {
                        format!("'{}' is {} day(s) overdue!", todo.title, days_overdue)
                    } else {
                        format!("'{}' is {} hour(s) overdue!", todo.title, hours_overdue)
                    };
                    
                    reminders.push(Reminder {
                        message,
                        emoji: "🚨".to_string(),
                        priority: ReminderPriority::Critical,
                    });
                }
                // Due today
                else if time_diff < Duration::days(1) {
                    let hours_left = time_diff.num_hours();
                    let message = if hours_left < 1 {
                        format!("'{}' is due in less than an hour!", todo.title)
                    } else {
                        format!("'{}' is due in {} hour(s)!", todo.title, hours_left)
                    };
                    
                    reminders.push(Reminder {
                        message,
                        emoji: "⏰".to_string(),
                        priority: ReminderPriority::Warning,
                    });
                }
                // Due tomorrow
                else if time_diff < Duration::days(2) {
                    reminders.push(Reminder {
                        message: format!("'{}' is due tomorrow!", todo.title),
                        emoji: "📅".to_string(),
                        priority: ReminderPriority::Info,
                    });
                }
                // Due within a week
                else if time_diff < Duration::days(7) {
                    let days_left = time_diff.num_days();
                    reminders.push(Reminder {
                        message: format!("'{}' is due in {} day(s)!", todo.title, days_left),
                        emoji: "📋".to_string(),
                        priority: ReminderPriority::Info,
                    });
                }
            }
        }
        
        // Check for todos without due dates that are old
        for todo in todos.iter().filter(|t| t.status == Status::Pending && t.due_date.is_none()) {
            let age = now.signed_duration_since(todo.created_at);
            
            if age > Duration::days(7) {
                let days_old = age.num_days();
                reminders.push(Reminder {
                    message: format!("'{}' has been pending for {} day(s) - consider setting a due date!", todo.title, days_old),
                    emoji: "💭".to_string(),
                    priority: ReminderPriority::Info,
                });
            }
        }
        
        // Sort reminders by priority (Critical first, then Warning, then Info)
        reminders.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
        
        reminders
    }
    
    pub fn get_daily_summary(&self, todos: &[Todo]) -> String {
        let pending_count = todos.iter().filter(|t| t.status == Status::Pending).count();
        let completed_today = todos.iter()
            .filter(|t| {
                t.status == Status::Completed && 
                t.updated_at.date_naive() == Local::now().date_naive()
            })
            .count();
        
        let now = Local::now();
        let due_today = todos.iter()
            .filter(|t| {
                t.status == Status::Pending &&
                t.due_date.map_or(false, |due| {
                    let due_datetime = DateTime::<Local>::from_naive_utc_and_offset(due, *now.offset());
                    due_datetime.date_naive() == now.date_naive()
                })
            })
            .count();
        
        let overdue = todos.iter()
            .filter(|t| {
                t.status == Status::Pending &&
                t.due_date.map_or(false, |due| {
                    let due_datetime = DateTime::<Local>::from_naive_utc_and_offset(due, *now.offset());
                    due_datetime < now
                })
            })
            .count();
        
        format!(
            "📊 Daily Summary: {} pending, {} completed today, {} due today, {} overdue",
            pending_count, completed_today, due_today, overdue
        )
    }
}