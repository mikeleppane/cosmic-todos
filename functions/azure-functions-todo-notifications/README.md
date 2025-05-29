# Azure Functions Todo Notifications

This project is an Azure Functions application designed to manage todo notifications. It utilizes the SendGrid Email API to send notifications to assignees based on the status and due dates of todo items.

## Project Structure

```
azure-functions-todo-notifications
├── function_app.py               # Entry point for the Azure Functions application
├── functions
│   ├── __init__.py               # Marks the functions directory as a package
│   ├── scheduled_notifications.py  # Function that runs on a schedule to send notifications
│   └── todo_trigger.py            # Function that triggers on new or updated todo items
├── shared
│   ├── __init__.py               # Marks the shared directory as a package
│   ├── email_service.py           # Logic for sending emails using SendGrid
│   ├── notification_logic.py      # Logic for determining when to send reminders
│   └── models.py                  # Data models for the application
├── requirements.txt               # Lists dependencies for the project
├── host.json                      # Configuration settings for the Azure Functions host
├── local.settings.json            # Local settings including connection strings and API keys
└── README.md                      # Documentation for the project
```

## Features

1. **Scheduled Notifications**: The application runs a function twice a day (at midnight and noon) to send notification emails to todo assignees.
2. **Immediate Notifications**: When a new todo item is created or updated, the assignee is notified immediately.
3. **Reminder Logic**: 
   - Sends a reminder email one day before the due date.
   - Sends reminders on the due date based on the time.
   - If the todo item status is "NotStarted" and it is overdue, reminders are sent continuously.

## Setup Instructions

1. Clone the repository:
   ```
   git clone <repository-url>
   cd azure-functions-todo-notifications
   ```

2. Install the required dependencies:
   ```
   pip install -r requirements.txt
   ```

3. Configure your SendGrid API key in `local.settings.json`.

4. Deploy the Azure Functions application to Azure.

## Usage

- The scheduled function will automatically run based on the defined schedule.
- The todo trigger function will respond to changes in the todo items, sending notifications as specified.

## Contributing

Contributions are welcome! Please submit a pull request or open an issue for any enhancements or bug fixes.