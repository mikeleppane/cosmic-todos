import logging
import os
from datetime import datetime, timedelta

import pytz
from azure.cosmos import CosmosClient
from azure.cosmos.exceptions import CosmosResourceNotFoundError


class TodoService:
    def __init__(
        self,
        cosmos_endpoint=None,
        cosmos_key=None,
        database_name="familyleppanen",
        container_name="todos",
    ):
        # Use environment variables if not provided
        self.cosmos_endpoint = cosmos_endpoint or os.getenv("COSMOS_DB_ENDPOINT")
        self.cosmos_key = cosmos_key or os.getenv("COSMOS_DB_KEY")
        self.database_name = database_name
        self.container_name = container_name

        if not self.cosmos_endpoint or not self.cosmos_key:
            raise ValueError("Cosmos DB endpoint and key must be provided")

        # Initialize Cosmos client
        self.client = CosmosClient(self.cosmos_endpoint, self.cosmos_key)
        self.database = self.client.get_database_client(self.database_name)
        self.container = self.database.get_container_client(self.container_name)

    def upsert_item(self, todo):
        """Insert or update a todo item in Cosmos DB"""
        try:
            # Ensure the item has required fields
            if not todo.get("id"):
                raise ValueError("Todo item must have an 'id' field")

            # Add metadata for tracking
            current_time = datetime.now(pytz.utc)

            # Set created_at if it's a new item (doesn't exist)
            if not todo.get("created_at"):
                todo["created_at"] = current_time.timestamp()

            # Always update the modified_at timestamp
            todo["updated_at"] = current_time.timestamp()

            # Upsert the item in Cosmos DB
            result = self.container.upsert_item(body=todo)

            logging.info(f"Successfully upserted todo item with ID: {todo['id']}")
            return result

        except ValueError as ve:
            logging.error(f"Validation error upserting todo: {str(ve)}")
            raise
        except Exception as e:
            logging.error(
                f"Error upserting todo item {todo.get('id', 'unknown')}: {str(e)}"
            )
            raise

    def get_assignee_email(self, todo_item):
        return todo_item.get("email", "")

    def is_due_soon(self, due_date):
        if isinstance(due_date, str):
            due_date = datetime.fromisoformat(due_date.replace("Z", "+00:00"))
        return due_date <= datetime.now(pytz.utc) + timedelta(days=1)

    def is_overdue(self, due_date):
        if isinstance(due_date, str):
            due_date = datetime.fromisoformat(due_date.replace("Z", "+00:00"))
        return due_date < datetime.now(pytz.utc)

    def get_all_todos(self):
        """Retrieve all todo items from Cosmos DB"""
        try:
            query = "SELECT * FROM c"
            items = list(
                self.container.query_items(
                    query=query, enable_cross_partition_query=True
                )
            )
            return items
        except Exception as e:
            logging.error(f"Error retrieving todos: {str(e)}")
            return []

    def get_todo_by_id(self, todo_id, partition_key=None):
        """Retrieve a specific todo item by ID"""
        try:
            if partition_key:
                item = self.container.read_item(
                    item=todo_id, partition_key=partition_key
                )
            else:
                # If no partition key provided, query for the item
                query = f"SELECT * FROM c WHERE c.id = '{todo_id}'"
                items = list(
                    self.container.query_items(
                        query=query, enable_cross_partition_query=True
                    )
                )
                item = items[0] if items else None
            return item
        except CosmosResourceNotFoundError:
            logging.warning(f"Todo item with ID {todo_id} not found")
            return None
        except Exception as e:
            logging.error(f"Error retrieving todo {todo_id}: {str(e)}")
            return None

    def get_todos_by_status(self, status):
        """Retrieve todo items by status"""
        try:
            query = f"SELECT * FROM c WHERE c.status = '{status}'"
            items = list(
                self.container.query_items(
                    query=query, enable_cross_partition_query=True
                )
            )
            return items
        except Exception as e:
            logging.error(f"Error retrieving todos by status {status}: {str(e)}")
            return []

    def get_due_items(self):
        """Get overdue items that are not started"""
        try:
            current_time = datetime.now(pytz.utc).isoformat()
            query = f"""
                SELECT * FROM c 
                WHERE c.status = 'Not Started' 
                AND c.due_date < '{current_time}'
            """
            items = list(
                self.container.query_items(
                    query=query, enable_cross_partition_query=True
                )
            )
            return items
        except Exception as e:
            logging.error(f"Error retrieving due items: {str(e)}")
            return []

    def get_items_due_soon(self):
        """Get items due within the next 24 hours"""
        try:
            current_time = datetime.now(pytz.utc)
            future_time = current_time + timedelta(days=1)

            query = f"""
                SELECT * FROM c 
                WHERE c.due_date >= '{current_time.isoformat()}' 
                AND c.due_date <= '{future_time.isoformat()}'
                AND c.status != 'Completed'
            """
            items = list(
                self.container.query_items(
                    query=query, enable_cross_partition_query=True
                )
            )
            return items
        except Exception as e:
            logging.error(f"Error retrieving items due soon: {str(e)}")
            return []

    def get_todos_by_assignee(self, email):
        """Get todos assigned to a specific email"""
        try:
            query = f"SELECT * FROM c WHERE c.email = '{email}'"
            items = list(
                self.container.query_items(
                    query=query, enable_cross_partition_query=True
                )
            )
            return items
        except Exception as e:
            logging.error(f"Error retrieving todos for assignee {email}: {str(e)}")
            return []

    def notify_assignee(self, todo_item):
        email = self.get_assignee_email(todo_item)
        # Logic to send email using email_service
        return f"Notification sent to {email}"

    def handle_new_or_updated_item(self, todo_item):
        if todo_item.get("status") != "Not Started":
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
