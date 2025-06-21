use anyhow::{Context, Result, anyhow};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::storage::Storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct AuthManager {
    storage: Storage,
    current_session: Option<Session>,
    users: HashMap<String, User>,
}

impl AuthManager {
    pub fn new(storage: &Storage) -> Result<Self> {
        let users = storage.load_users()?;
        let current_session = storage.load_session()?;
        
        Ok(Self {
            storage: storage.clone(),
            current_session,
            users,
        })
    }
    
    pub async fn register(&mut self, username: &str, email: &str, password: &str) -> Result<User> {
        // Check if username already exists
        if self.users.values().any(|u| u.username == username) {
            return Err(anyhow!("Username already exists"));
        }
        
        // Check if email already exists
        if self.users.values().any(|u| u.email == email) {
            return Err(anyhow!("Email already exists"));
        }
        
        // Validate input
        if username.trim().is_empty() {
            return Err(anyhow!("Username cannot be empty"));
        }
        
        if email.trim().is_empty() || !email.contains('@') {
            return Err(anyhow!("Invalid email address"));
        }
        
        if password.len() < 6 {
            return Err(anyhow!("Password must be at least 6 characters long"));
        }
        
        // Hash password
        let password_hash = hash(password, DEFAULT_COST)
            .context("Failed to hash password")?;
        
        // Create user
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            created_at: Utc::now(),
            last_login: None,
        };
        
        // Store user
        self.users.insert(user.id.clone(), user.clone());
        self.storage.save_users(&self.users)?;
        
        Ok(user)
    }
    
    pub async fn login(&mut self, username: &str, password: &str) -> Result<User> {
        let user = self.users.values()
            .find(|u| u.username == username)
            .ok_or_else(|| anyhow!("Invalid username or password"))?;
        
        if !verify(password, &user.password_hash)
            .context("Failed to verify password")? {
            return Err(anyhow!("Invalid username or password"));
        }
        
        // Update last login
        let mut updated_user = user.clone();
        updated_user.last_login = Some(Utc::now());
        self.users.insert(updated_user.id.clone(), updated_user.clone());
        self.storage.save_users(&self.users)?;
        
        // Create session
        let session = Session {
            user_id: updated_user.id.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::days(7), // Session expires in 7 days
        };
        
        self.current_session = Some(session.clone());
        self.storage.save_session(&session)?;
        
        Ok(updated_user)
    }
    
    pub async fn logout(&mut self) -> Result<()> {
        self.current_session = None;
        self.storage.clear_session()?;
        Ok(())
    }
    
    pub fn is_authenticated(&self) -> bool {
        if let Some(ref session) = self.current_session {
            session.expires_at > Utc::now()
        } else {
            false
        }
    }
    
    pub fn get_current_user(&self) -> Result<User> {
        let session = self.current_session.as_ref()
            .ok_or_else(|| anyhow!("Not authenticated"))?;
        
        if session.expires_at <= Utc::now() {
            return Err(anyhow!("Session expired"));
        }
        
        let user = self.users.get(&session.user_id)
            .ok_or_else(|| anyhow!("User not found"))?;
        
        Ok(user.clone())
    }
    
    pub fn get_user_by_id(&self, user_id: &str) -> Option<&User> {
        self.users.get(user_id)
    }
}