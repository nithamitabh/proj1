use anyhow::{Result};
use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};
use colored::*;
use dialoguer::{Input, Password, Select};

mod auth;
mod todo;
mod storage;
mod reminder;

use auth::AuthManager;
use todo::{Todo, TodoManager, Priority, Status};
use storage::Storage;
use reminder::ReminderService;

#[derive(Parser)]
#[command(name = "todo")]
#[command(about = "A CLI todo application with user authentication")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a new user
    Register,
    /// Login to your account
    Login,
    /// Logout from current session
    Logout,
    /// Add a new todo item
    Add {
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        priority: Option<String>,
        #[arg(short = 'd', long)]
        due_date: Option<String>,
    },
    /// List all todos
    List {
        #[arg(short, long)]
        status: Option<String>,
        #[arg(short, long)]
        priority: Option<String>,
    },
    /// Complete a todo
    Complete {
        id: Option<String>,
    },
    /// Delete a todo
    Delete {
        id: Option<String>,
    },
    /// Edit a todo
    Edit {
        id: Option<String>,
    },
    /// Show overdue todos
    Overdue,
    /// Show today's todos
    Today,
    /// Check for reminders
    Reminders,
    /// Show user status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let mut app = TodoApp::new()?;
    
    match &cli.command {
        Some(Commands::Register) => app.register().await?,
        Some(Commands::Login) => app.login().await?,
        Some(Commands::Logout) => app.logout().await?,
        Some(Commands::Add { title, description, priority, due_date }) => {
            app.ensure_authenticated()?;
            app.add_todo(title.clone(), description.clone(), priority.clone(), due_date.clone()).await?;
        },
        Some(Commands::List { status, priority }) => {
            app.ensure_authenticated()?;
            app.list_todos(status.clone(), priority.clone()).await?;
        },
        Some(Commands::Complete { id }) => {
            app.ensure_authenticated()?;
            app.complete_todo(id.clone()).await?;
        },
        Some(Commands::Delete { id }) => {
            app.ensure_authenticated()?;
            app.delete_todo(id.clone()).await?;
        },
        Some(Commands::Edit { id }) => {
            app.ensure_authenticated()?;
            app.edit_todo(id.clone()).await?;
        },
        Some(Commands::Overdue) => {
            app.ensure_authenticated()?;
            app.show_overdue().await?;
        },
        Some(Commands::Today) => {
            app.ensure_authenticated()?;
            app.show_today().await?;
        },
        Some(Commands::Reminders) => {
            app.ensure_authenticated()?;
            app.check_reminders().await?;
        },
        Some(Commands::Status) => {
            app.show_status().await?;
        },
        None => {
            app.interactive_mode().await?;
        }
    }
    
    Ok(())
}

struct TodoApp {
    auth_manager: AuthManager,
    todo_manager: TodoManager,
    storage: Storage,
    reminder_service: ReminderService,
}

impl TodoApp {
    fn new() -> Result<Self> {
        let storage = Storage::new()?;
        let auth_manager = AuthManager::new(&storage)?;
        let todo_manager = TodoManager::new(&storage)?;
        let reminder_service = ReminderService::new();
        
        Ok(Self {
            auth_manager,
            todo_manager,
            storage,
            reminder_service,
        })
    }
    
    async fn register(&mut self) -> Result<()> {
        println!("{}", "🚀 Welcome to Todo CLI - Registration".bright_cyan().bold());
        
        let username: String = Input::new()
            .with_prompt("Username")
            .interact_text()?;
            
        let email: String = Input::new()
            .with_prompt("Email")
            .interact_text()?;
            
        let password = Password::new()
            .with_prompt("Password")
            .with_confirmation("Confirm password", "Passwords don't match")
            .interact()?;
            
        match self.auth_manager.register(&username, &email, &password).await {
            Ok(_) => {
                println!("{} Registration successful! You can now login.", "✅".green());
            },
            Err(e) => {
                println!("{} Registration failed: {}", "❌".red(), e);
            }
        }
        
        Ok(())
    }
    
    async fn login(&mut self) -> Result<()> {
        println!("{}", "🔐 Login to Todo CLI".bright_blue().bold());
        
        let username: String = Input::new()
            .with_prompt("Username")
            .interact_text()?;
            
        let password = Password::new()
            .with_prompt("Password")
            .interact()?;
            
        match self.auth_manager.login(&username, &password).await {
            Ok(user) => {
                println!("{} Welcome back, {}! 👋", "✅".green(), user.username.bright_green());
                self.check_reminders().await?;
            },
            Err(e) => {
                println!("{} Login failed: {}", "❌".red(), e);
            }
        }
        
        Ok(())
    }
    
    async fn logout(&mut self) -> Result<()> {
        self.auth_manager.logout().await?;
        println!("{} Logged out successfully! 👋", "✅".green());
        Ok(())
    }
    
    fn ensure_authenticated(&self) -> Result<()> {
        if !self.auth_manager.is_authenticated() {
            println!("{} Please login first using: todo login", "❌".red());
            std::process::exit(1);
        }
        Ok(())
    }
    
    async fn add_todo(&mut self, title: Option<String>, description: Option<String>, priority: Option<String>, due_date: Option<String>) -> Result<()> {
        let current_user = self.auth_manager.get_current_user()?;
        
        let title = match title {
            Some(t) => t,
            None => Input::new()
                .with_prompt("Todo title")
                .interact_text()?,
        };
        
        let description = match description {
            Some(d) => Some(d),
            None => {
                let desc: String = Input::new()
                    .with_prompt("Description (optional)")
                    .allow_empty(true)
                    .interact_text()?;
                if desc.is_empty() { None } else { Some(desc) }
            }
        };
        
        let priority = match priority {
            Some(p) => Priority::from_string(&p)?,
            None => {
                let priorities = ["Low", "Medium", "High"];
                let selection = Select::new()
                    .with_prompt("Priority")
                    .default(1)
                    .items(&priorities)
                    .interact()?;
                match selection {
                    0 => Priority::Low,
                    1 => Priority::Medium,
                    2 => Priority::High,
                    _ => Priority::Medium,
                }
            }
        };
        
        let due_date = match due_date {
            Some(d) => Some(chrono::NaiveDateTime::parse_from_str(&format!("{} 23:59:59", d), "%Y-%m-%d %H:%M:%S")?),
            None => {
                let date_str: String = Input::new()
                    .with_prompt("Due date (YYYY-MM-DD, optional)")
                    .allow_empty(true)
                    .interact_text()?;
                if date_str.is_empty() {
                    None
                } else {
                    Some(chrono::NaiveDateTime::parse_from_str(&format!("{} 23:59:59", date_str), "%Y-%m-%d %H:%M:%S")?)
                }
            }
        };
        
        let todo = Todo::new(title, description, priority, due_date, current_user.id.clone());
        self.todo_manager.add_todo(todo.clone()).await?;
        
        println!("{} Todo added successfully!", "✅".green());
        self.print_todo(&todo);
        
        Ok(())
    }
    
    async fn list_todos(&self, status_filter: Option<String>, priority_filter: Option<String>) -> Result<()> {
        let current_user = self.auth_manager.get_current_user()?;
        let todos = self.todo_manager.get_user_todos(&current_user.id).await?;
        
        let filtered_todos: Vec<&Todo> = todos.iter()
            .filter(|todo| {
                if let Some(ref status) = status_filter {
                    let filter_status = Status::from_string(status).unwrap_or(Status::Pending);
                    todo.status == filter_status
                } else {
                    true
                }
            })
            .filter(|todo| {
                if let Some(ref priority) = priority_filter {
                    let filter_priority = Priority::from_string(priority).unwrap_or(Priority::Medium);
                    todo.priority == filter_priority
                } else {
                    true
                }
            })
            .collect();
        
        if filtered_todos.is_empty() {
            println!("{} No todos found!", "ℹ️".blue());
            return Ok(());
        }
        
        println!("\n{}", "📋 Your Todos".bright_cyan().bold());
        println!("{}", "─".repeat(80).bright_black());
        
        for todo in filtered_todos {
            self.print_todo(todo);
            println!();
        }
        
        Ok(())
    }
    
    async fn complete_todo(&mut self, id: Option<String>) -> Result<()> {
        let current_user = self.auth_manager.get_current_user()?;
        
        let todo_id = match id {
            Some(id) => id,
            None => {
                let todos = self.todo_manager.get_user_todos(&current_user.id).await?;
                let pending_todos: Vec<&Todo> = todos.iter()
                    .filter(|t| t.status == Status::Pending)
                    .collect();
                
                if pending_todos.is_empty() {
                    println!("{} No pending todos found!", "ℹ️".blue());
                    return Ok(());
                }
                
                let items: Vec<String> = pending_todos.iter()
                    .map(|t| format!("{} - {}", t.id[..8].to_string(), t.title))
                    .collect();
                
                let selection = Select::new()
                    .with_prompt("Select todo to complete")
                    .items(&items)
                    .interact()?;
                
                pending_todos[selection].id.clone()
            }
        };
        
        self.todo_manager.complete_todo(&todo_id).await?;
        println!("{} Todo completed! 🎉", "✅".green());
        
        Ok(())
    }
    
    async fn delete_todo(&mut self, id: Option<String>) -> Result<()> {
        let current_user = self.auth_manager.get_current_user()?;
        
        let todo_id = match id {
            Some(id) => id,
            None => {
                let todos = self.todo_manager.get_user_todos(&current_user.id).await?;
                
                if todos.is_empty() {
                    println!("{} No todos found!", "ℹ️".blue());
                    return Ok(());
                }
                
                let items: Vec<String> = todos.iter()
                    .map(|t| format!("{} - {}", t.id[..8].to_string(), t.title))
                    .collect();
                
                let selection = Select::new()
                    .with_prompt("Select todo to delete")
                    .items(&items)
                    .interact()?;
                
                todos[selection].id.clone()
            }
        };
        
        self.todo_manager.delete_todo(&todo_id).await?;
        println!("{} Todo deleted!", "✅".green());
        
        Ok(())
    }
    
    async fn edit_todo(&mut self, id: Option<String>) -> Result<()> {
        let current_user = self.auth_manager.get_current_user()?;
        
        let todo_id = match id {
            Some(id) => id,
            None => {
                let todos = self.todo_manager.get_user_todos(&current_user.id).await?;
                
                if todos.is_empty() {
                    println!("{} No todos found!", "ℹ️".blue());
                    return Ok(());
                }
                
                let items: Vec<String> = todos.iter()
                    .map(|t| format!("{} - {}", t.id[..8].to_string(), t.title))
                    .collect();
                
                let selection = Select::new()
                    .with_prompt("Select todo to edit")
                    .items(&items)
                    .interact()?;
                
                todos[selection].id.clone()
            }
        };
        
        let mut todo = self.todo_manager.get_todo(&todo_id).await?;
        
        println!("Editing todo: {}", todo.title.bright_yellow());
        
        let new_title: String = Input::new()
            .with_prompt("Title")
            .default(todo.title.clone())
            .interact_text()?;
        
        let new_description: String = Input::new()
            .with_prompt("Description")
            .default(todo.description.clone().unwrap_or_default())
            .allow_empty(true)
            .interact_text()?;
        
        let priorities = ["Low", "Medium", "High"];
        let current_priority_index = match todo.priority {
            Priority::Low => 0,
            Priority::Medium => 1,
            Priority::High => 2,
        };
        
        let selection = Select::new()
            .with_prompt("Priority")
            .default(current_priority_index)
            .items(&priorities)
            .interact()?;
        
        let new_priority = match selection {
            0 => Priority::Low,
            1 => Priority::Medium,
            2 => Priority::High,
            _ => Priority::Medium,
        };
        
        todo.title = new_title;
        todo.description = if new_description.is_empty() { None } else { Some(new_description) };
        todo.priority = new_priority;
        todo.updated_at = chrono::Utc::now();
        
        self.todo_manager.update_todo(todo).await?;
        println!("{} Todo updated successfully!", "✅".green());
        
        Ok(())
    }
    
    async fn show_overdue(&self) -> Result<()> {
        let current_user = self.auth_manager.get_current_user()?;
        let todos = self.todo_manager.get_user_todos(&current_user.id).await?;
        
        let overdue_todos: Vec<&Todo> = todos.iter()
            .filter(|todo| {
                todo.status == Status::Pending && 
                todo.due_date.map_or(false, |due| {
                    let due_datetime = DateTime::<Local>::from_naive_utc_and_offset(due, *Local::now().offset());
                    due_datetime < Local::now()
                })
            })
            .collect();
        
        if overdue_todos.is_empty() {
            println!("{} No overdue todos! 🎉", "✅".green());
            return Ok(());
        }
        
        println!("\n{} {} Overdue Todos", "⚠️".red(), overdue_todos.len());
        println!("{}", "─".repeat(80).bright_black());
        
        for todo in overdue_todos {
            self.print_todo(todo);
            println!();
        }
        
        Ok(())
    }
    
    async fn show_today(&self) -> Result<()> {
        let current_user = self.auth_manager.get_current_user()?;
        let todos = self.todo_manager.get_user_todos(&current_user.id).await?;
        
        let today = Local::now().date_naive();
        let today_todos: Vec<&Todo> = todos.iter()
            .filter(|todo| {
                todo.due_date.map_or(false, |due| due.date() == today)
            })
            .collect();
        
        if today_todos.is_empty() {
            println!("{} No todos due today! 🎉", "ℹ️".blue());
            return Ok(());
        }
        
        println!("\n{} {} Todos Due Today", "📅".yellow(), today_todos.len());
        println!("{}", "─".repeat(80).bright_black());
        
        for todo in today_todos {
            self.print_todo(todo);
            println!();
        }
        
        Ok(())
    }
    
    async fn check_reminders(&self) -> Result<()> {
        let current_user = self.auth_manager.get_current_user()?;
        let todos = self.todo_manager.get_user_todos(&current_user.id).await?;
        
        let reminders = self.reminder_service.get_reminders(&todos);
        
        if !reminders.is_empty() {
            println!("\n{} You have {} reminders:", "🔔".bright_yellow(), reminders.len());
            for reminder in reminders {
                println!("  {} {}", reminder.emoji, reminder.message.bright_yellow());
            }
            println!();
        }
        
        Ok(())
    }
    
    async fn show_status(&self) -> Result<()> {
        if self.auth_manager.is_authenticated() {
            let current_user = self.auth_manager.get_current_user()?;
            let todos = self.todo_manager.get_user_todos(&current_user.id).await?;
            
            let pending = todos.iter().filter(|t| t.status == Status::Pending).count();
            let completed = todos.iter().filter(|t| t.status == Status::Completed).count();
            let overdue = todos.iter().filter(|t| {
                t.status == Status::Pending && 
                t.due_date.map_or(false, |due| {
                    let due_datetime = DateTime::<Local>::from_naive_utc_and_offset(due, *Local::now().offset());
                    due_datetime < Local::now()
                })
            }).count();
            
            println!("\n{} User Status", "👤".bright_blue());
            println!("Username: {}", current_user.username.bright_green());
            println!("Email: {}", current_user.email.bright_blue());
            println!("\n{} Todo Statistics", "📊".bright_cyan());
            println!("Pending: {}", pending.to_string().yellow());
            println!("Completed: {}", completed.to_string().green());
            println!("Overdue: {}", overdue.to_string().red());
            println!("Total: {}", todos.len().to_string().bright_white());
        } else {
            println!("{} Not logged in", "❌".red());
        }
        
        Ok(())
    }
    
    async fn interactive_mode(&mut self) -> Result<()> {
        println!("{}", "🚀 Welcome to Todo CLI".bright_cyan().bold());
        
        if !self.auth_manager.is_authenticated() {
            let options = ["Login", "Register", "Exit"];
            let selection = Select::new()
                .with_prompt("What would you like to do?")
                .items(&options)
                .interact()?;
                
            match selection {
                0 => self.login().await?,
                1 => self.register().await?,
                2 => return Ok(()),
                _ => return Ok(()),
            }
        }
        
        if self.auth_manager.is_authenticated() {
            self.check_reminders().await?;
            
            loop {
                let options = [
                    "Add Todo", "List Todos", "Complete Todo", "Edit Todo", 
                    "Delete Todo", "Show Overdue", "Show Today", "Status", "Logout", "Exit"
                ];
                
                let selection = Select::new()
                    .with_prompt("What would you like to do?")
                    .items(&options)
                    .interact()?;
                    
                match selection {
                    0 => self.add_todo(None, None, None, None).await?,
                    1 => self.list_todos(None, None).await?,
                    2 => self.complete_todo(None).await?,
                    3 => self.edit_todo(None).await?,
                    4 => self.delete_todo(None).await?,
                    5 => self.show_overdue().await?,
                    6 => self.show_today().await?,
                    7 => self.show_status().await?,
                    8 => {
                        self.logout().await?;
                        break;
                    },
                    9 => break,
                    _ => break,
                }
            }
        }
        
        Ok(())
    }
    
    fn print_todo(&self, todo: &Todo) {
        let status_emoji = match todo.status {
            Status::Pending => "⏳",
            Status::Completed => "✅",
        };
        
        let priority_emoji = match todo.priority {
            Priority::Low => "🟢",
            Priority::Medium => "🟡",
            Priority::High => "🔴",
        };
        
        let id_short = &todo.id[..8];
        
        println!("{} {} {} [{}] {}", 
            status_emoji, 
            priority_emoji,
            id_short.bright_black(),
            todo.title.bright_white().bold(),
            if todo.status == Status::Completed { "✨" } else { "" }
        );
        
        if let Some(description) = &todo.description {
            println!("   📝 {}", description.bright_black());
        }
        
        if let Some(due_date) = todo.due_date {
            let due_datetime = DateTime::<Local>::from_naive_utc_and_offset(due_date, *Local::now().offset());
            let is_overdue = due_datetime < Local::now() && todo.status == Status::Pending;
            
            if is_overdue {
                println!("   ⚠️  Due: {} {}", due_datetime.format("%Y-%m-%d %H:%M").to_string().red(), "(OVERDUE)".red().bold());
            } else {
                println!("   📅 Due: {}", due_datetime.format("%Y-%m-%d %H:%M").to_string().bright_blue());
            }
        }
        
        println!("   🕒 Created: {}", todo.created_at.format("%Y-%m-%d %H:%M").to_string().bright_black());
    }
}