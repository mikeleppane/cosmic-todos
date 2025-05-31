import os

import requests

EMAIL_SEND_API_KEY = os.environ.get("EMAIL_SEND_API_KEY")

FROM_EMAIL = "family.leppanen.todos@familyleppanen.net"


def send_email(to_email, subject, content):
    res = requests.post(
        "https://api.eu.mailgun.net/v3/familyleppanen.net/messages",
        auth=("api", EMAIL_SEND_API_KEY),
        data={
            "from": "family.leppanen.todos@familyleppanen.net",
            "to": to_email,
            "subject": subject,
            "text": content,
        },
    )
    if res.status_code == 200:
        return {
            "status": "success",
            "message": f"Email sent to {to_email} with subject '{subject}'",
        }
    else:
        return {
            "status": "error",
            "message": f"Failed to send email to {to_email}. Status code: {res.status_code}, Response: {res.text}",
        }


def notify_assignee(assignee_email, todo_item):
    subject = f"Notification for Todo Item: {todo_item['title']}"
    content = f"Hello,\n\nThis is a reminder for your todo item:\n\nTitle: {todo_item['title']}\nDue Date: {todo_item['due_date']}\nStatus: {todo_item['status']}\n\nBest regards,\nYour Todo App"
    return send_email(assignee_email, subject, content)
