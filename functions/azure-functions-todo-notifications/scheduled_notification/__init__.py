import logging
from datetime import datetime, timezone

import azure.functions as func
from shared.email_service import send_email
from shared.todo_service import TodoService


def main(mytimer: func.TimerRequest) -> None:
    """
    Timer-triggered Azure Function that runs every 30 minutes
    to send todo notifications based on due dates and completion status.
    """
    utc_timestamp = (
        datetime.now(tz=timezone.utc).replace(tzinfo=timezone.utc).isoformat()
    )

    if mytimer.past_due:
        logging.info("The timer is past due!")

    logging.info(f"Todo notification function executed at {utc_timestamp}")

    try:
        # Initialize the TodoService
        todo_service = TodoService()
        current_time = datetime.now(timezone.utc)

        # Process all active todos
        all_todos = todo_service.get_all_todos()

        for todo in all_todos:
            if todo.get("status") == "Completed":
                continue

            process_todo_notifications(todo, current_time, todo_service)

        logging.info("Todo notification processing completed successfully")

    except Exception as e:
        logging.error(f"Error in todo notification function: {str(e)}")
        raise


def process_todo_notifications(todo, current_time, todo_service):
    """Process notification logic for a single todo item"""
    try:
        due_date_epoch = todo.get("due_date")
        if not due_date_epoch:
            return

        # Parse due date from epoch time
        try:
            # Handle both integer and string epoch timestamps
            if isinstance(due_date_epoch, str):
                due_date_epoch = float(due_date_epoch)

            # Convert epoch to UTC datetime
            due_date = datetime.fromtimestamp(due_date_epoch, tz=timezone.utc)
        except (ValueError, TypeError) as e:
            logging.error(
                f'Invalid due_date format for todo {todo.get("id", "unknown")}: {due_date_epoch}'
            )
            return

        time_diff = due_date - current_time
        cast_floats_fields_to_int(todo)

        # Check if item is overdue (up to 1 hour)
        if (
            time_diff.total_seconds() < 0 and abs(time_diff.total_seconds()) <= 3600
        ):  # 1 hour = 3600 seconds
            send_overdue_notification(todo, due_date)

        # Check for 24-hour reminder (23.5 to 24.5 hours before due)
        elif 24 * 3600 <= time_diff.total_seconds() < 25 * 3600:
            if not has_received_24h_reminder(todo):
                send_24h_reminder(todo, due_date)
                mark_24h_reminder_sent(todo, todo_service)

        # Check for final reminder (0.5 to 23.5 hours before due)
        elif 0.5 * 3600 <= time_diff.total_seconds() <= 23.5 * 3600:
            if not has_received_final_reminder(todo):
                send_final_reminder(todo, due_date)
                mark_final_reminder_sent(todo, todo_service)

        # Daily reminders for items that are overdue by more than 1 hour
        elif time_diff.total_seconds() < -3600:  # More than 1 hour overdue
            if should_send_daily_reminder(todo, current_time):
                send_daily_overdue_reminder(todo, due_date)
                update_last_notification_time(todo, current_time, todo_service)

    except Exception as e:
        logging.error(f'Error processing todo {todo.get("id", "unknown")}: {str(e)}')


def cast_floats_fields_to_int(todo):
    """Convert float fields in todo to int for consistency"""
    try:
        for key in ["due_date", "created_at", "updated_at", "last_notification_time"]:
            if key in todo and isinstance(todo[key], float):
                todo[key] = int(todo[key])
    except Exception as e:
        logging.error(f"Error casting float fields: {str(e)}")


def send_overdue_notification(todo, due_date):
    """Send notification for items that are overdue by at most 1 hour"""
    try:
        email = todo.get("email")
        subject = f"Todo Item Overdue: {todo.get('title', 'Untitled')}"
        body = f"""
        Hello {todo.get('assignee')}\n\n,
        Your todo item "{todo.get('title', 'Untitled')}" is now overdue.
        
        Due Date: {due_date.strftime('%Y-%m-%d %H:%M:%S UTC')}
        Description: {todo.get('description', 'No description')}
        
        Please complete this task as soon as possible.
        """

        send_email(email, subject, body)
        logging.info(f'Sent overdue notification to {email} for todo {todo.get("id")}')

    except Exception as e:
        logging.error(f"Failed to send overdue notification: {str(e)}")


def send_24h_reminder(todo, due_date):
    """Send 24-hour advance reminder"""
    try:
        email = todo.get("email")
        name = todo.get("assignee")
        subject = f"Reminder: Todo Due in 24 Hours - {todo.get('title', 'Untitled')}"
        body = f"""
        Hello {name},\n\n
        This is a reminder that your todo item is due in approximately 24 hours.
        
        Title: {todo.get('title', 'Untitled')}
        Due Date: {due_date.strftime('%Y-%m-%d %H:%M:%S UTC')}
        Description: {todo.get('description', 'No description')}
        
        Please plan to complete this task soon.
        """

        send_email(email, subject, body)
        logging.info(f'Sent 24h reminder to {email} for todo {todo.get("id")}')

    except Exception as e:
        logging.error(f"Failed to send 24h reminder: {str(e)}")


def send_final_reminder(todo, due_date):
    """Send final reminder when less than 24 hours remain"""
    try:
        email = todo.get("email")
        subject = f"Final Reminder: Todo Due Soon - {todo.get('title', 'Untitled')}"
        body = f"""
        Hello {todo.get('assignee')},\n\n
        This is your final reminder - your todo item is due very soon!
        
        Title: {todo.get('title', 'Untitled')}
        Due Date: {due_date.strftime('%Y-%m-%d %H:%M:%S UTC')}
        Description: {todo.get('description', 'No description')}
        
        Please complete this task immediately.
        """

        send_email(email, subject, body)
        logging.info(f'Sent final reminder to {email} for todo {todo.get("id")}')

    except Exception as e:
        logging.error(f"Failed to send final reminder: {str(e)}")


def send_daily_overdue_reminder(todo, due_date):
    """Send daily reminder for overdue items"""
    try:
        email = todo.get("email")
        subject = f"Daily Reminder: Overdue Todo - {todo.get('title', 'Untitled')}"
        body = f"""
        Hello {todo.get('assignee')},\n\n
        Daily reminder: Your todo item is still overdue and needs attention.
        
        Title: {todo.get('title', 'Untitled')}
        Due Date: {due_date.strftime('%Y-%m-%d %H:%M:%S UTC')}
        Description: {todo.get('description', 'No description')}
        
        Please complete this overdue task.
        """

        send_email(email, subject, body)
        logging.info(
            f'Sent daily overdue reminder to {email} for todo {todo.get("id")}'
        )

    except Exception as e:
        logging.error(f"Failed to send daily overdue reminder: {str(e)}")


def has_received_24h_reminder(todo):
    """Check if 24-hour reminder has already been sent"""
    return todo.get("reminder_24h_sent", False)


def has_received_final_reminder(todo):
    """Check if final reminder has already been sent"""
    return todo.get("final_reminder_sent", False)


def should_send_daily_reminder(todo, current_time):
    """Check if daily reminder should be sent (every 24 hours)"""
    last_notification_epoch = todo.get("last_notification_time")
    if not last_notification_epoch:
        return True

    try:
        # Handle both integer and string epoch timestamps
        if isinstance(last_notification_epoch, str):
            last_notification_epoch = float(last_notification_epoch)

        last_notification_time = datetime.fromtimestamp(
            last_notification_epoch, tz=timezone.utc
        )
        time_since_last = current_time - last_notification_time

        # Send daily reminder if more than 24 hours since last notification
        return time_since_last.total_seconds() >= 24 * 3600

    except (ValueError, TypeError) as e:
        logging.error(
            f"Invalid last_notification_time format: {last_notification_epoch}"
        )
        return True  # Default to sending if we can't parse the timestamp


def mark_24h_reminder_sent(todo, todo_service):
    """Mark that 24-hour reminder has been sent"""
    try:
        # Update the todo item in Cosmos DB
        todo["reminder_24h_sent"] = True
        todo_service.upsert_item(todo)
        logging.info(f'Marked 24h reminder as sent for todo {todo.get("id")}')

    except Exception as e:
        logging.error(f"Failed to mark 24h reminder as sent: {str(e)}")


def mark_final_reminder_sent(todo, todo_service):
    """Mark that final reminder has been sent"""
    try:
        # Update the todo item in Cosmos DB
        todo["final_reminder_sent"] = True
        todo_service.upsert_item(todo)
        logging.info(f'Marked final reminder as sent for todo {todo.get("id")}')

    except Exception as e:
        logging.error(f"Failed to mark final reminder as sent: {str(e)}")


def update_last_notification_time(todo, current_time, todo_service):
    """Update the last notification time for daily reminders"""
    try:
        # Store as epoch timestamp for consistency
        todo["last_notification_time"] = int(current_time.timestamp())
        todo_service.upsert_item(todo)
        logging.info(f'Updated last notification time for todo {todo.get("id")}')

    except Exception as e:
        logging.error(f"Failed to update last notification time: {str(e)}")
