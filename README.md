# Todo CLI 🚀

A comprehensive command-line todo application written in Rust with user authentication, emoji support, and markdown export functionality.

## Features ✨

- 🔐 **User Authentication**: Register and login system with secure password hashing
- 📝 **Todo Management**: Add, edit, complete, and delete todos
- 🎨 **Rich CLI Interface**: Colorful output with emojis for better user experience
- 📅 **Due Date Tracking**: Set due dates and get reminded about overdue tasks
- 🔔 **Smart Reminders**: Automatic notifications for overdue and upcoming tasks
- 📊 **Priority System**: High, Medium, Low priority levels
- 📋 **Markdown Export**: All todos are automatically saved to a markdown file
- 💾 **Data Persistence**: User data and todos stored in JSON format
- 🏠 **Local Storage**: All data stored in `~/.todo-cli/` directory

## Installation 🛠️

1. **Prerequisites**: Make sure you have Rust installed. If not, install it from [rustup.rs](https://rustup.rs/)

2. **Clone and Build**:
   ```bash
   # Build the project
   cargo build --release
   
   # Install to system (optional)
   cargo install --path .
   ```

3. **Run the application**:
   ```bash
   cargo run
   # or if installed:
   todo-cli
   ```

## Usage 📖

### Interactive Mode
Simply run the application without arguments to enter interactive mode:
```bash
cargo run
```

### Command Line Interface

#### Authentication
```bash
# Register a new user
cargo run register

# Login
cargo run login

# Logout
cargo run logout

# Check user status
cargo run status
```

#### Todo Management
```bash
# Add a new todo
cargo run add --title "Complete project" --description "Finish the Rust CLI project" --priority high --due-date 2024-12-31

# List all todos
cargo run list

# List todos by status
cargo run list --status pending
cargo run list --status completed

# List todos by priority
cargo run list --priority high

# Complete a todo
cargo run complete [todo-id]

# Edit a todo
cargo run edit [todo-id]

# Delete a todo
cargo run delete [todo-id]
```

#### Viewing Todos
```bash
# Show overdue todos
cargo run overdue

# Show today's todos
cargo run today

# Check reminders
cargo run reminders
```

## File Structure 📁

```
~/.todo-cli/
├── users.json      # User accounts and authentication data
├── todos.json      # All todo items
├── session.json    # Current user session
└── todos.md        # Markdown export of all todos
```

## Data Storage 💾

- **Users**: Stored in JSON format with bcrypt-hashed passwords
- **Todos**: Stored in JSON format with full metadata
- **Sessions**: Temporary session data for authentication
- **Markdown**: Human-readable export of all todos with proper formatting

## Emojis and Colors 🎨

The application uses a rich set of emojis and colors to enhance the user experience:

- 🟢 Low Priority
- 🟡 Medium Priority  
- 🔴 High Priority
- ⏳ Pending Status
- ✅ Completed Status
- 🚨 Overdue Reminder
- ⏰ Due Soon Reminder
- 📅 Due Date Information
- 🔔 Reminder Notifications

## Reminder System 🔔

The application provides intelligent reminders:

- **Critical**: Overdue tasks (🚨)
- **Warning**: Due today or within hours (⏰)
- **Info**: Due tomorrow or within a week (📅)
- **Maintenance**: Old tasks without due dates (💭)

## Future Enhancements 🚀

The application is designed to support future features:

- Database integration (PostgreSQL, SQLite)
- Cloud synchronization
- Team collaboration
- Mobile app integration
- Advanced filtering and search
- Task dependencies
- Time tracking
- Recurring tasks

## Security 🔒

- Passwords are securely hashed using bcrypt
- Session management with expiration
- Local data storage (no cloud dependencies)
- User isolation (users can only see their own todos)

## Contributing 🤝

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License 📄

This project is open source and available under the MIT License.

## Support 💬

If you encounter any issues or have questions, please create an issue in the repository.

---

**Happy Todo Managing!** 🎉