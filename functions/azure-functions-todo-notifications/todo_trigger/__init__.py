import json
import logging
import os
from datetime import datetime, timezone
from typing import Any, Dict

import azure.functions as func
from shared.email_service import send_email
from shared.todo_service import TodoService


def main(documents: func.DocumentList) -> None:
    """
    Cosmos DB trigger that processes todo document changes
    and sends appropriate notifications.
    """
    if documents:
        logging.info(f"Processing {len(documents)} todo document changes")

        for document in documents:
            try:
                doc_dict = json.loads(document.to_json())
                process_todo_change(doc_dict)
            except Exception as e:
                logging.error(f"Error processing document: {str(e)}")
                continue


def process_todo_change(todo_doc: Dict[str, Any]) -> None:
    """Process a single todo document change"""
    try:
        todo_id = todo_doc.get("id", "unknown")
        logging.info(f"Processing todo change for ID: {todo_id}")

        # Check if this is a system-generated update (should be ignored)
        if is_system_update(todo_doc):
            logging.info(f"Skipping system update for todo {todo_id}")
            return

        # Check if this is a new todo (doesn't have notification tracking fields)
        is_new_todo = is_new_todo_document(todo_doc)

        # Check if this is a notification-only update (should be ignored)
        is_notification_update = is_notification_field_update(todo_doc)

        if is_new_todo:
            send_new_todo_notification(todo_doc)
            # Mark immediately using synchronous approach
            mark_new_todo_notification_sent(todo_doc)
        elif not is_notification_update:
            send_todo_update_notification(todo_doc)

    except Exception as e:
        logging.error(f"Error in process_todo_change: {str(e)}")


def is_system_update(todo_doc: Dict[str, Any]) -> bool:
    """
    Check if this is a system-generated update that should not trigger notifications.
    Uses improved detection logic for Azure Functions.
    """
    # Check for notification tracking fields that indicate system updates
    tracking_fields = [
        "new_todo_notification_sent",
        "reminder_24h_sent",
        "final_reminder_sent",
        "last_notification_time",
    ]

    # Count tracking fields present
    tracking_count = sum(
        1 for field in tracking_fields if todo_doc.get(field) is not None
    )

    # If we have the new_todo_notification_sent field set to True,
    # this is likely the update we made to mark notification as sent
    if todo_doc.get("new_todo_notification_sent") is True:
        # Check if this update happened very recently after creation
        created_at = todo_doc.get("created_at")
        updated_at = todo_doc.get("updated_at")

        if created_at and updated_at:
            try:
                if isinstance(created_at, (int, float)) and isinstance(
                    updated_at, (int, float)
                ):
                    created_time = datetime.fromtimestamp(created_at, tz=timezone.utc)
                    updated_time = datetime.fromtimestamp(updated_at, tz=timezone.utc)

                    # If updated within 30 seconds of creation and has notification flag,
                    # this is our system update
                    time_diff = (updated_time - created_time).total_seconds()
                    if 0 < time_diff < 30:
                        logging.info(
                            f"Detected system update (notification marking) for todo {todo_doc.get('id')}"
                        )
                        return True
            except (ValueError, TypeError) as e:
                logging.warning(f"Error parsing timestamps: {str(e)}")

    # Additional check: if document has multiple tracking fields but few content fields
    content_fields = [
        "title",
        "description",
        "assignee",
        "email",
        "due_date",
        "priority",
        "status",
    ]
    content_count = sum(
        1 for field in content_fields if todo_doc.get(field) is not None
    )

    # If we have more tracking fields than content, likely a system update
    if tracking_count > 0 and content_count < 3:
        return True

    return False


def is_new_todo_document(todo_doc: Dict[str, Any]) -> bool:
    """
    Determine if this is a new todo by checking notification tracking fields
    """
    # If new_todo_notification_sent is already True, this is not a new todo
    if todo_doc.get("new_todo_notification_sent") is True:
        return False

    # Check if this appears to be a genuinely new todo
    notification_fields = [
        "reminder_24h_sent",
        "final_reminder_sent",
        "last_notification_time",
        "new_todo_notification_sent",
    ]

    # If any notification field is set, it's not a new todo
    has_notification_fields = any(
        todo_doc.get(field) is not None and todo_doc.get(field) is not False
        for field in notification_fields
    )

    if has_notification_fields:
        return False

    # Additional validation: check if recently created
    created_at = todo_doc.get("created_at")
    if created_at:
        try:
            if isinstance(created_at, (int, float)):
                created_time = datetime.fromtimestamp(created_at, tz=timezone.utc)
                current_time = datetime.now(timezone.utc)
                time_since_creation = (current_time - created_time).total_seconds()

                # If created within last 60 seconds, it's likely a new todo
                if time_since_creation < 60:
                    logging.info(
                        f"Detected new todo created {time_since_creation:.1f} seconds ago: {todo_doc.get('id')}"
                    )
                    return True
        except (ValueError, TypeError) as e:
            logging.warning(f"Error parsing created_at timestamp: {str(e)}")

    # Default to not new if we can't determine
    return False


def is_notification_field_update(todo_doc: Dict[str, Any]) -> bool:
    """
    Check if this update only involves notification tracking fields
    """
    # This is handled by is_system_update now for better accuracy
    return False


def send_new_todo_notification(todo_doc: Dict[str, Any]) -> None:
    """Send notification for a newly created todo"""
    try:
        email = todo_doc.get("email")
        assignee = todo_doc.get("assignee") or "User"
        title = todo_doc.get("title", "Untitled")

        if not email:
            logging.warning(f"No email found for new todo {todo_doc.get('id')}")
            return

        # Format due date if available
        due_date_str = "Not specified"
        due_date_epoch = todo_doc.get("due_date")
        if due_date_epoch:
            try:
                if isinstance(due_date_epoch, str):
                    due_date_epoch = float(due_date_epoch)
                due_date = datetime.fromtimestamp(due_date_epoch, tz=timezone.utc)
                due_date_str = due_date.strftime("%A, %B %d, %Y at %I:%M %p UTC")
            except (ValueError, TypeError):
                logging.warning(
                    f"Invalid due_date format for todo {todo_doc.get('id')}"
                )

        subject = f"New Todo Assigned: {title}"
        body = f"""Hello {assignee},

A new todo item has been assigned to you:

📋 Title: {title}
📝 Description: {todo_doc.get('description', 'No description')}
📅 Due Date: {due_date_str}

Please review and plan accordingly.

Best regards,
Family Leppänen Todo System"""

        send_email(email, subject, body)
        logging.info(
            f"✅ Sent new todo notification to {email} for todo {todo_doc.get('id')}"
        )

    except Exception as e:
        logging.error(f"❌ Failed to send new todo notification: {str(e)}")


def send_todo_update_notification(todo_doc: Dict[str, Any]) -> None:
    """Send notification for todo updates (excluding notification field updates)"""
    try:
        email = todo_doc.get("email")
        assignee = todo_doc.get("assignee") or "User"
        title = todo_doc.get("title", "Untitled")

        if not email:
            logging.warning(f"No email found for updated todo {todo_doc.get('id')}")
            return

        # Format due date if available
        due_date_str = "Not specified"
        due_date_epoch = todo_doc.get("due_date")
        if due_date_epoch:
            try:
                if isinstance(due_date_epoch, str):
                    due_date_epoch = float(due_date_epoch)
                due_date = datetime.fromtimestamp(due_date_epoch, tz=timezone.utc)
                due_date_str = due_date.strftime("%A, %B %d, %Y at %I:%M %p UTC")
            except (ValueError, TypeError):
                logging.warning(
                    f"Invalid due_date format for todo {todo_doc.get('id')}"
                )

        subject = f"Todo Updated: {title}"
        body = f"""Hello {assignee},

Your todo item has been updated:

📋 Title: {title}
📝 Description: {todo_doc.get('description', 'No description')}
📅 Due Date: {due_date_str}
✅ Status: {todo_doc.get('status', 'Pending')}

Please review the changes.

Best regards,
Family Leppänen Todo System"""

        send_email(email, subject, body)
        logging.info(
            f"✅ Sent todo update notification to {email} for todo {todo_doc.get('id')}"
        )

    except Exception as e:
        logging.error(f"❌ Failed to send todo update notification: {str(e)}")


def mark_new_todo_notification_sent(todo_doc: Dict[str, Any]) -> None:
    """
    Mark that new todo notification has been sent.
    Uses synchronous approach suitable for Azure Functions.
    """
    try:
        todo_service = TodoService()
        todo_id = todo_doc.get("id")

        if not todo_id:
            logging.error("Cannot mark notification sent - no todo ID found")
            return

        # Get the current document from Cosmos DB to ensure we have latest version
        try:
            current_doc = todo_service.get_todo_by_id(
                todo_id, partition_key="family_todos"
            )
            if not current_doc:
                logging.error(f"Could not retrieve todo {todo_id} from database")
                return
        except Exception as e:
            logging.error(f"Error retrieving todo {todo_id}: {str(e)}")
            # Fallback to using the provided document
            current_doc = todo_doc.copy()

        # Set the notification flag
        current_doc["new_todo_notification_sent"] = True

        # Update the updated_at timestamp
        current_doc["updated_at"] = int(datetime.now(timezone.utc).timestamp())

        # Update in Cosmos DB
        result = todo_service.upsert_item(current_doc)

        if result:
            logging.info(
                f"✅ Successfully marked new todo notification as sent for todo {todo_id}"
            )
        else:
            logging.error(f"❌ Failed to update todo {todo_id} in database")

    except Exception as e:
        logging.error(f"❌ Failed to mark new todo notification as sent: {str(e)}")
        # Log the full exception for debugging
        import traceback

        logging.error(f"Full traceback: {traceback.format_exc()}")
