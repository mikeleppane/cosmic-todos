from .email_service import send_email
from .todo_service import get_assignee_email, get_due_date_reminders


def notify_assignee(todo_item):
    assignee_email = get_assignee_email(todo_item)
    if assignee_email:
        send_email(
            assignee_email,
            "Todo Item Notification",
            f"You have a new notification for your todo item: {todo_item}",
        )


def schedule_reminders(todo_items):
    reminders = get_due_date_reminders(todo_items)
    for reminder in reminders:
        assignee_email = get_assignee_email(reminder)
        if assignee_email:
            send_email(
                assignee_email,
                "Reminder: Todo Item Due Soon",
                f"Reminder for your todo item: {reminder}",
            )
