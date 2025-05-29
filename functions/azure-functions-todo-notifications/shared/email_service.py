import os

from sendgrid import SendGridAPIClient
from sendgrid.helpers.mail import Mail

SENDGRID_API_KEY = os.environ.get("SENDGRID_API_KEY")


def send_email(to_email, subject, content):
    message = Mail(
        from_email="familyleppanen02@gmail.com",  # Replace with your verified SendGrid sender email
        to_emails=to_email,
        subject=subject,
        plain_text_content=content,
    )

    try:
        sg = SendGridAPIClient(SENDGRID_API_KEY)
        response = sg.send(message)
        return response.status_code
    except Exception as e:
        print(f"Error sending email: {e}")
        return None


def notify_assignee(assignee_email, todo_item):
    subject = f"Notification for Todo Item: {todo_item['title']}"
    content = f"Hello,\n\nThis is a reminder for your todo item:\n\nTitle: {todo_item['title']}\nDue Date: {todo_item['due_date']}\nStatus: {todo_item['status']}\n\nBest regards,\nYour Todo App"
    return send_email(assignee_email, subject, content)
