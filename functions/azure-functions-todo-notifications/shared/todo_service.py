from datetime import datetime, timedelta

import pytz


class TodoService:
    def __init__(self, db):
        self.db = db

    def get_assignee_email(self, todo_item):
        return todo_item["email"]

    def is_due_soon(self, due_date):
        return due_date <= datetime.now(pytz.utc) + timedelta(days=1)

    def is_overdue(self, due_date):
        return due_date < datetime.now(pytz.utc)

    def get_due_items(self):
        return [
            item
            for item in self.db
            if item["status"] == "Not Started" and self.is_overdue(item["due_date"])
        ]

    def get_items_due_soon(self):
        return [item for item in self.db if self.is_due_soon(item["due_date"])]

    def notify_assignee(self, todo_item):
        email = self.get_assignee_email(todo_item)
        # Logic to send email using email_service
        return f"Notification sent to {email}"

    def handle_new_or_updated_item(self, todo_item):
        if todo_item["status"] != "Not Started":
            return
        self.notify_assignee(todo_item)

    def handle_due_notifications(self):
        due_items = self.get_due_items()
        for item in due_items:
            self.notify_assignee(item)

    def handle_reminders(self):
        due_soon_items = self.get_items_due_soon()
        for item in due_soon_items:
            self.notify_assignee(item)
