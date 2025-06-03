import logging
from datetime import datetime, timedelta

import azure.functions as func
from shared.email_service import send_email


def main(documents: func.DocumentList) -> None:
    logging.info(f"Processing {len(documents)} changed documents from Cosmos DB")

    for document in documents:
        assignee = document.get("assignee")
        due_date = document.get("due_date")
        status = document.get("status")
        created_at = document.get("created_at")
        updated_at = document.get("updated_at")
        document["is_new"] = False
        document["is_updated"] = False
        due_date_as_datetime = datetime.fromtimestamp(due_date)
        # include week day and time in the due date
        due_date_formatted = due_date_as_datetime.strftime("%A, %B %d, %Y at %I:%M %p")

        # determine if the todo item is new or updated, item does not have is_new or is_updated fields
        if created_at and updated_at:
            if int(updated_at) > int(created_at):
                document["is_updated"] = True
            else:
                document["is_new"] = True

        # Check if the assignee is provided
        if not assignee:
            logging.warning(
                f"Todo item {document.get('id')} does not have an assignee. Skipping notification."
            )
            continue

        # Get the assignee's email
        assignee_email = document.get("email", "mleppan23@gmail.com")
        assignee_name = document.get("assignee", "Mikko")

        # Notify immediately on creation or update
        if document.get("is_new"):
            content = (
                f"Hello {assignee_name},\n\n"
                f"A new todo item has been created: {document.get('title')}.\n"
                f" Description: {document.get('description', 'No description provided')}.\n"
                f" Due date: {due_date_formatted}.\n"
                f" Assignee: {assignee}.\n\n"
                "\n\nPlease check your todo list for more details.\n\n"
            )

            send_email(
                assignee_email,
                "New Todo Item Created",
                content,
            )

        if document.get("is_updated"):
            content = (
                f"Hello {assignee_name},\n\n"
                f"Your todo item has been updated: {document.get('title')}.\n"
                f" Description: {document.get('description', 'No description provided')}."
                f" Due date: {due_date_formatted}.\n"
                f" Assignee: {assignee}.\n"
                "\n\nPlease check your todo list for more details.\n\n"
            )
            send_email(
                assignee_email,
                "Todo Item Updated",
                content,
            )

        # Check for reminders based on due date and status
        if due_date:
            today = datetime.now()

            # Send reminder one day before due date
            if (
                due_date_as_datetime - timedelta(days=1)
            ).date() == today.date() and status != "Completed":
                send_email(
                    assignee_email,
                    f"Todo Due Tomorrow Reminder",
                    f"Hello {assignee_name},\n\nYour todo item '{document.get('title')}' is due tomorrow.\n Please complete it soon.\n\n",
                )

            # Send reminder if overdue and status is NotStarted
            if today > due_date_as_datetime and status == "Pending":
                send_email(
                    assignee_email,
                    "Todo Overdue Reminder",
                    f"Hello {assignee_name},\n\nYour todo item '{document.get('title')}' is overdue.\n Please take action. ",
                )
