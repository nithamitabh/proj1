use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::storage::Storage;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Pending,
    Completed,
}

impl Status {
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "pending" | "p" => Ok(Status::Pending),
            "completed" | "complete" | "done" | "c" => Ok(Status::Completed),
            _ => Err(anyhow!("Invalid status: {}. Use 'pending' or 'completed'", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "low" | "l" => Ok(Priority::Low),
            "medium" | "med" | "m" => Ok(Priority::Medium),
            "high" | "h" => Ok(Priority::High),
            _ => Err(anyhow!("Invalid priority: {}. Use 'low', 'medium', or 'high'", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
    pub priority: Priority,
    pub due_date: Option<NaiveDateTime>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: String,
}

impl Todo {
    pub fn new(
        title: String,
        description: Option<String>,
        priority: Priority,
        due_date: Option<NaiveDateTime>,
        user_id: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            status: Status::Pending,
            priority,
            due_date,
            created_at: now,
            updated_at: now,
            user_id,
        }
    }
}

pub struct TodoManager {
    storage: Storage,
    todos: HashMap<String, Todo>,
}

impl TodoManager {
    pub fn new(storage: &Storage) -> Result<Self> {
        let todos = storage.load_todos()?;
        Ok(Self {
            storage: storage.clone(),
            todos,
        })
    }

    pub async fn add_todo(&mut self, todo: Todo) -> Result<()> {
        self.todos.insert(todo.id.clone(), todo.clone());
        self.storage.save_todos(&self.todos)?;
        self.storage.append_to_markdown(&todo)?;
        Ok(())
    }

    pub async fn get_user_todos(&self, user_id: &str) -> Result<Vec<Todo>> {
        Ok(self.todos.values()
            .filter(|todo| todo.user_id == user_id)
            .cloned()
            .collect())
    }

    pub async fn get_todo(&self, todo_id: &str) -> Result<Todo> {
        self.todos.get(todo_id)
            .cloned()
            .ok_or_else(|| anyhow!("Todo not found"))
    }

    pub async fn complete_todo(&mut self, todo_id: &str) -> Result<()> {
        // Scope the mutable borrow so it ends before we use `todo` again
        let updated_todo = {
            let todo = self.todos.get_mut(todo_id)
                .ok_or_else(|| anyhow!("Todo not found"))?;
            todo.status = Status::Completed;
            todo.updated_at = Utc::now();
            todo.clone() // Clone so borrow ends here
        };

        self.storage.save_todos(&self.todos)?;
        self.storage.update_markdown_todo(&updated_todo)?;
        Ok(())
    }

    pub async fn update_todo(&mut self, updated_todo: Todo) -> Result<()> {
        self.todos.insert(updated_todo.id.clone(), updated_todo.clone());
        self.storage.save_todos(&self.todos)?;
        self.storage.update_markdown_todo(&updated_todo)?;
        Ok(())
    }

    pub async fn delete_todo(&mut self, todo_id: &str) -> Result<()> {
        // Remove first so mutable borrow ends early
        let removed = self.todos.remove(todo_id)
            .ok_or_else(|| anyhow!("Todo not found"))?;

        self.storage.save_todos(&self.todos)?;
        self.storage.remove_from_markdown(&removed)?;
        Ok(())
    }
}
