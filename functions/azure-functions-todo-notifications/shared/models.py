class TodoItem:
    def __init__(self, id, title, assignee, due_date, status):
        self.id = id
        self.title = title
        self.assignee = assignee
        self.due_date = due_date
        self.status = status

    def is_overdue(self):
        from datetime import datetime

        return self.due_date < datetime.now()

    def is_due_soon(self):
        from datetime import datetime, timedelta

        return (
            self.due_date <= datetime.now() + timedelta(days=1)
            and self.status == "NotStarted"
        )
