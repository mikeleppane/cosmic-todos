import logging
import os
from datetime import datetime, timedelta, timezone

import azure.functions as func
from shared.email_service import send_email
from shared.todo_service import get_assignee_email, get_due_todos


def main(mytimer: func.TimerRequest) -> None:
    logging.info("Scheduled notification function triggered.")

    # Get current date and time
    now = datetime.now(tz=timezone.utc)
    logging.info(f"Current UTC time: {now}")

    # Get todos that are due for reminders
    due_todos = get_due_todos(now)

    for todo in due_todos:
        assignee_email = get_assignee_email(todo.assignee)
        if assignee_email:
            send_email(assignee_email, todo)
            logging.info(f"Email sent to {assignee_email} for todo item: {todo.id}")
        else:
            logging.warning(f"No email found for assignee: {todo.assignee}")
