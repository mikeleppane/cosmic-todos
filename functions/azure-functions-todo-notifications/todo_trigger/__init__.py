import logging
from datetime import datetime, timedelta

import azure.functions as func
from shared.email_service import send_email
from shared.todo_service import get_assignee_email, get_todo_items


def main(req: func.HttpRequest) -> func.HttpResponse:
    logging.info("Processing a new or updated todo item.")

    # Get the todo item details from the request
    todo_item = req.get_json()
    assignee = todo_item.get("assignee")
    due_date = todo_item.get("due_date")
    status = todo_item.get("status")
    created_at = todo_item.get("created_at")
    updated_at = todo_item.get("updated_at")
    todo_item["is_new"] = False
    todo_item["is_updated"] = False

    # determine if the todo item is new or updated, item does not have is_new or is_updated fields
    if created_at and updated_at:
        if int(updated_at) > int(created_at):
            todo_item["is_updated"] = True
        else:
            todo_item["is_new"] = True

    # Check if the assignee is provided
    if not assignee:
        return func.HttpResponse("Assignee is required.", status_code=400)

    # Get the assignee's email
    assignee_email = get_assignee_email(assignee)

    # Notify immediately on creation or update
    if todo_item.get("is_new"):
        content = (
            f"A new todo item has been created: {todo_item.get('title')}."
            f" Description: {todo_item.get('description', 'No description provided')}."
            f" Due date: {todo_item.get('due_date', 'No due date set')}."
            f" Assignee: {assignee}."
        )
        send_email(
            assignee_email,
            "Todo Item Notification",
            f"You have a new todo item: {todo_item.get('title')}.",
        )

    if todo_item.get("is_updated"):
        content = (
            f"Your todo item has been updated: {todo_item.get('title')}."
            f" Description: {todo_item.get('description', 'No description provided')}."
            f" Due date: {todo_item.get('due_date', 'No due date set')}."
            f" Assignee: {assignee}."
        )
        send_email(
            assignee_email,
            "Todo Item Updated",
            content,
        )

    # Check for reminders based on due date and status
    if due_date:
        due_date_obj = datetime.strptime(due_date, "%Y-%m-%d")
        today = datetime.now()

        # Send reminder one day before due date
        if (
            due_date_obj - timedelta(days=1)
        ).date() == today.date() and status != "Completed":
            send_email(
                assignee_email,
                "Todo Reminder",
                f"Reminder: Your todo item '{todo_item.get('title')}' is due tomorrow.",
            )

        # Send reminder if overdue and status is NotStarted
        if today.date() > due_date_obj.date() and status == "NotStarted":
            send_email(
                assignee_email,
                "Overdue Todo Reminder",
                f"Your todo item '{todo_item.get('title')}' is overdue.",
            )

    return func.HttpResponse("Notification processed.", status_code=200)
