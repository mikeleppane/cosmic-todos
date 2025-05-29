use std::str::FromStr;

use crate::app::{create_todo_server, delete_todo_server, get_todos_server, update_todo_server};
use crate::todo::{Todo, TodoAssignee, TodoStatus};
use chrono::{Datelike, Local, NaiveDate, TimeZone};
use leptos::web_sys;
use leptos::{ev, prelude::*};

#[component]
#[allow(clippy::too_many_lines)]
#[allow(clippy::must_use_candidate)]
pub fn HomePage() -> impl IntoView {
    // State for the todo list
    let (todos, set_todos) = signal(Vec::<Todo>::new());

    // Loading and error states
    let (loading, set_loading) = signal(true);
    let (error_message, set_error_message) = signal(String::new());

    // Modal state for creating/editing todos
    let (show_modal, set_show_modal) = signal(false);
    let (editing_todo, set_editing_todo) = signal(None::<Todo>);

    // Calendar state
    let (current_month, set_current_month) = signal(Local::now().month());
    let (current_year, set_current_year) = signal(Local::now().year());
    let today = Local::now().date_naive();

    // Form fields for new/edit todo
    let (new_title, set_new_title) = signal(String::new());
    let (new_description, set_new_description) = signal(String::new());
    let (new_due_date, set_new_due_date) = signal(String::new());
    let (new_due_time, set_new_due_time) = signal(String::new());
    let (new_assignee, set_new_assignee) = signal("Mikko".to_string());
    let (new_status, set_new_status) = signal("Not Started".to_string());

    // Helper to reset form
    let reset_form = move || {
        set_new_title.set(String::new());
        set_new_description.set(String::new());
        set_new_due_date.set(String::new());
        set_new_due_time.set(String::new());
        set_new_assignee.set("Mikko".to_string());
        set_new_status.set("Not Started".to_string());
        set_editing_todo.set(None);
    };

    // Helper to populate form with existing todo data
    let populate_form = move |todo: &Todo| {
        set_new_title.set(todo.title.clone());
        set_new_description.set(todo.description.clone().unwrap_or_default());
        set_new_assignee.set(todo.assignee.as_str().to_string());
        set_new_status.set(todo.status.as_str().to_string());

        if let Some(timestamp) = todo.due_date {
            if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp, 0) {
                let local_datetime = datetime.with_timezone(&chrono::Local);
                set_new_due_date.set(local_datetime.format("%Y-%m-%d").to_string());
                set_new_due_time.set(local_datetime.format("%H:%M").to_string());
            }
        } else {
            set_new_due_date.set(String::new());
            set_new_due_time.set(String::new());
        }
    };

    // ...existing calendar helper functions...
    let get_month_name = |month: u32| -> &'static str {
        match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Invalid",
        }
    };

    let get_days_in_month = |year: i32, month: u32| -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    };

    let get_first_day_of_month = |year: i32, month: u32| -> u32 {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, 1) {
            date.weekday().num_days_from_sunday()
        } else {
            0
        }
    };

    // Calendar navigation
    let prev_month = move |_| {
        if current_month.get() == 1 {
            set_current_month.set(12);
            set_current_year.update(|y| *y -= 1);
        } else {
            set_current_month.update(|m| *m -= 1);
        }
    };

    let next_month = move |_| {
        if current_month.get() == 12 {
            set_current_month.set(1);
            set_current_year.update(|y| *y += 1);
        } else {
            set_current_month.update(|m| *m += 1);
        }
    };

    // Actions
    let load_todos_action = Action::new(move |(): &()| async move { get_todos_server().await });
    let create_todo_action = Action::new(move |todo: &Todo| {
        let todo = todo.clone();
        async move { create_todo_server(todo).await }
    });
    let update_todo_action = Action::new(move |todo: &Todo| {
        let todo = todo.clone();
        async move { update_todo_server(todo).await }
    });
    let delete_todo_action = Action::new(move |id: &String| {
        let id = id.clone();
        async move { delete_todo_server(id).await }
    });

    // Load todos on component mount
    Effect::new(move |_| {
        load_todos_action.dispatch(());
    });

    // Watch for load todos results
    Effect::new(move |_| {
        if let Some(result) = load_todos_action.value().get() {
            match result {
                Ok(todos_list) => {
                    set_todos.set(todos_list);
                    set_loading.set(false);
                    set_error_message.set(String::new());
                }
                Err(e) => {
                    set_error_message.set(format!("Failed to load todos: {e}"));
                    set_loading.set(false);
                }
            }
        }
    });

    // Watch for create todo results
    Effect::new(move |_| {
        if let Some(result) = create_todo_action.value().get() {
            match result {
                Ok(created_todo) => {
                    set_todos.update(|todos| {
                        todos.push(created_todo);
                    });
                    reset_form();
                    set_show_modal.set(false);
                    set_error_message.set(String::new());
                }
                Err(e) => {
                    set_error_message.set(format!("Failed to create todo: {e}"));
                }
            }
        }
    });

    // Watch for update todo results
    Effect::new(move |_| {
        if let Some(result) = update_todo_action.value().get() {
            match result {
                Ok(updated_todo) => {
                    set_todos.update(|todos| {
                        if let Some(index) = todos.iter().position(|t| t.id == updated_todo.id) {
                            todos[index] = updated_todo;
                        }
                    });
                    reset_form();
                    set_show_modal.set(false);
                    set_error_message.set(String::new());
                }
                Err(e) => {
                    set_error_message.set(format!("Failed to update todo: {e}"));
                }
            }
        }
    });

    // Watch for delete todo results
    Effect::new(move |_| {
        if let Some(result) = delete_todo_action.value().get() {
            match result {
                Ok(_) => {
                    // Reload todos after successful delete
                    load_todos_action.dispatch(());
                    set_error_message.set(String::new());
                }
                Err(e) => {
                    set_error_message.set(format!("Failed to delete todo: {e}"));
                }
            }
        }
    });

    // Handle form submission
    let handle_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let title = new_title.get_untracked();
        if title.trim().is_empty() {
            set_error_message.set("Title is required".to_string());
            return;
        }

        let due_timestamp = if new_due_date.get_untracked().is_empty() {
            None
        } else {
            let date_str = new_due_date.get_untracked();
            let time_str = if new_due_time.get_untracked().is_empty() {
                "00:00"
            } else {
                &new_due_time.get_untracked()
            };
            let datetime_str = format!("{date_str} {time_str}");

            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M") {
                Some(
                    chrono::Local
                        .from_local_datetime(&dt)
                        .single()
                        .map_or(0, |dt| dt.timestamp()),
                )
            } else {
                set_error_message.set("Invalid date/time format".to_string());
                return;
            }
        };

        let todo = Todo {
            id: editing_todo.get_untracked().map_or(0, |t| t.id),
            title: title.trim().to_string(),
            description: if new_description.get_untracked().trim().is_empty() {
                None
            } else {
                Some(new_description.get_untracked().trim().to_string())
            },
            due_date: due_timestamp,
            assignee: TodoAssignee::from_str(&new_assignee.get_untracked())
                .unwrap_or(TodoAssignee::Mikko),
            status: TodoStatus::from_str(&new_status.get_untracked())
                .unwrap_or(TodoStatus::NotStarted),
        };

        set_error_message.set(String::new());

        if editing_todo.get_untracked().is_some() {
            update_todo_action.dispatch(todo);
        } else {
            create_todo_action.dispatch(todo);
        }
    };

    let is_creating = move || create_todo_action.pending().get();
    let is_updating = move || update_todo_action.pending().get();
    let is_deleting = move || delete_todo_action.pending().get();

    let format_due_date = |timestamp: i64| -> String {
        if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp, 0) {
            datetime.format("%B %d, %Y at %I:%M %p").to_string()
        } else {
            "Invalid date".to_string()
        }
    };

    view! {
        <main class="my-0 mx-auto max-w-6xl p-6 min-h-screen">
            // Error message display
            <Show when=move || !error_message.get().is_empty()>
                <div class="mb-4 p-3 rounded-xl bg-red-50 border border-red-100 shadow-sm">
                    <p class="text-sm font-medium text-red-600">
                        {move || error_message.get()}
                    </p>
                </div>
            </Show>

            // Header with create button
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-3xl font-bold bg-gradient-to-r from-purple-600 to-fuchsia-600 bg-clip-text text-transparent">
                    "Family Todos"
                </h1>
                <button
                    on:click=move |_| {
                        reset_form();
                        set_show_modal.set(true);
                    }
                    class="px-4 py-2 bg-gradient-to-r from-purple-500 to-fuchsia-500 text-white rounded-lg hover:from-purple-600 hover:to-fuchsia-600 transition-all duration-200 shadow-lg"
                >
                    "Add Todo"
                </button>
            </div>

            // Main content grid
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Calendar section (keeping existing calendar code)
                <div class="lg:col-span-1">
                    <div class="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
                        <div class="flex justify-between items-center mb-4">
                            <h2 class="text-xl font-semibold text-gray-800">
                                {move || format!("{} {}", get_month_name(current_month.get()), current_year.get())}
                            </h2>
                            <div class="flex gap-2">
                                <button
                                    on:click=prev_month
                                    class="p-2 hover:bg-gray-100 rounded-lg transition-colors"
                                >
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/>
                                    </svg>
                                </button>
                                <button
                                    on:click=next_month
                                    class="p-2 hover:bg-gray-100 rounded-lg transition-colors"
                                >
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                    </svg>
                                </button>
                            </div>
                        </div>

                        // Calendar grid
                        <div class="grid grid-cols-7 gap-1 mb-2">
                            // Day headers
                            {["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].iter().map(|day| {
                                view! {
                                    <div class="p-2 text-center text-xs font-medium text-gray-500">
                                        {*day}
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>

                        <div class="grid grid-cols-7 gap-1">
                            {move || {
                                let year = current_year.get();
                                let month = current_month.get();
                                let days_in_month = get_days_in_month(year, month);
                                let first_day = get_first_day_of_month(year, month);
                                let mut calendar_days = Vec::new();

                                // Empty cells for days before the first day of the month
                                for _ in 0..first_day {
                                    calendar_days.push(view! {
                                        <div class="p-2 h-8">{String::new()}</div>
                                    });
                                }

                                // Days of the month
                                for day in 1..=days_in_month {
                                    let is_today = if let Some(current_date) = NaiveDate::from_ymd_opt(year, month, day) {
                                        current_date == today
                                    } else {
                                        false
                                    };

                                    if is_today {
                                        calendar_days.push(view! {
                                            <div class="p-2 h-8 text-center text-sm rounded-lg bg-gradient-to-r from-purple-500 to-fuchsia-500 text-white font-semibold">
                                                {format!("{day}")}
                                            </div>
                                        });
                                    } else {
                                        calendar_days.push(view! {
                                            <div class="p-2 h-8 text-center text-sm rounded-lg hover:bg-gray-100 cursor-pointer transition-colors">
                                                {format!("{day}")}
                                            </div>
                                        });
                                    }
                                }
                                calendar_days
                            }}
                        </div>

                        // Today's date display
                        <div class="mt-4 pt-4 border-t border-gray-100">
                            <p class="text-sm text-gray-600 text-center">
                                "Today: "
                                <span class="font-medium text-purple-600">
                                    {today.format("%B %d, %Y").to_string()}
                                </span>
                            </p>
                        </div>
                    </div>
                </div>

                // Todo list section with edit/delete buttons
                <div class="lg:col-span-2">
                    <Show when=move || loading.get()>
                        <div class="flex justify-center items-center py-8">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600"></div>
                            <span class="ml-2 text-gray-600">"Loading todos..."</span>
                        </div>
                    </Show>

                    <Show when=move || !loading.get()>
                        <div class="space-y-4">
                            {move || {
                                let todos_list = todos.get();
                                if todos_list.is_empty() {
                                    view! {
                                        <div class="text-center py-12 bg-white rounded-2xl shadow-sm border border-gray-100">
                                            <div class="text-gray-400 mb-4">
                                                <svg class="mx-auto h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"/>
                                                </svg>
                                            </div>
                                            <h3 class="text-lg font-medium text-gray-900 mb-2">"No todos yet"</h3>
                                            <p class="text-gray-500 mb-4">"Create your first todo to get started!"</p>
                                            <button
                                                on:click=move |_| {
                                                    reset_form();
                                                    set_show_modal.set(true);
                                                }
                                                class="px-4 py-2 bg-gradient-to-r from-purple-500 to-fuchsia-500 text-white rounded-lg hover:from-purple-600 hover:to-fuchsia-600 transition-all duration-200"
                                            >
                                                "Create First Todo"
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="grid gap-4">
                                            {todos_list.into_iter().map(|todo| {
                                                let todo_clone = todo.clone();
                                                let todo_id = todo.id;

                                                let status_color = match todo.status {
                                                    TodoStatus::NotStarted => "bg-gray-100 text-gray-800",
                                                    TodoStatus::Completed => "bg-green-100 text-green-800",
                                                };

                                                let assignee_color = match todo.assignee {
                                                    TodoAssignee::Mikko => "bg-purple-100 text-purple-800",
                                                    TodoAssignee::Niina => "bg-pink-100 text-pink-800",
                                                };

                                                view! {
                                                    <div class="bg-white rounded-xl shadow-sm border border-gray-100 p-6 hover:shadow-md transition-shadow duration-200">
                                                        <div class="flex justify-between items-start mb-3">
                                                            <h3 class="text-lg font-semibold text-gray-900">{todo.title.clone()}</h3>
                                                            <div class="flex items-center gap-2">
                                                                <span class={format!("px-2 py-1 text-xs font-medium rounded-full {status_color}")}>
                                                                    {todo.status.as_str()}
                                                                </span>
                                                                <div class="flex gap-1">
                                                                    <button
                                                                        on:click=move |_| {
                                                                            populate_form(&todo_clone);
                                                                            set_editing_todo.set(Some(todo_clone.clone()));
                                                                            set_show_modal.set(true);
                                                                        }
                                                                        class="p-1 text-gray-500 hover:text-blue-600 hover:bg-blue-50 rounded transition-colors"
                                                                        title="Edit todo"
                                                                    >
                                                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                                                                        </svg>
                                                                    </button>
                                                                    <button
                                                                        on:click=move |_| {
                                                                            if web_sys::window()
                                                                                .unwrap()
                                                                                .confirm_with_message("Are you sure you want to delete this todo?")
                                                                                .unwrap_or(false)
                                                                            {
                                                                                delete_todo_action.dispatch(todo_id.to_string());
                                                                            }
                                                                        }
                                                                        class="p-1 text-gray-500 hover:text-red-600 hover:bg-red-50 rounded transition-colors"
                                                                        title="Delete todo"
                                                                        disabled=is_deleting
                                                                    >
                                                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                                                                        </svg>
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        </div>

                                                        {todo.description.as_ref().map(|desc| view! {
                                                            <p class="text-gray-600 mb-3">{desc.clone()}</p>
                                                        })}

                                                        <div class="flex flex-wrap gap-2 items-center">
                                                            <span class={format!("px-2 py-1 text-xs font-medium rounded-full {assignee_color}")}>
                                                                {todo.assignee.as_str()}
                                                            </span>

                                                            {todo.due_date.map(|timestamp| view! {
                                                                <span class="px-2 py-1 text-xs font-medium rounded-full bg-yellow-100 text-yellow-800">
                                                                    {format!("Due: {}", format_due_date(timestamp))}
                                                                </span>
                                                            })}
                                                        </div>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            }}
                        </div>
                    </Show>
                </div>
            </div>

            // Modal for creating/editing todos
            <Show when=move || show_modal.get()>
                <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
                    <div class="bg-white rounded-2xl p-6 w-full max-w-md shadow-2xl">
                        <div class="flex justify-between items-center mb-4">
                            <h2 class="text-xl font-bold text-gray-800">
                                {move || if editing_todo.get().is_some() { "Edit Todo" } else { "Create New Todo" }}
                            </h2>
                            <button
                                on:click=move |_| set_show_modal.set(false)
                                class="text-gray-500 hover:text-gray-700 text-2xl leading-none"
                            >
                                "×"
                            </button>
                        </div>

                        <form on:submit=handle_submit>
                            // Title field
                            <div class="mb-4">
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Title *"
                                </label>
                                <input
                                    type="text"
                                    prop:value=move || new_title.get()
                                    on:input=move |ev| set_new_title.set(event_target_value(&ev))
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                    placeholder="Enter todo title"
                                    required
                                />
                            </div>

                            // Description field
                            <div class="mb-4">
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Description"
                                </label>
                                <textarea
                                    prop:value=move || new_description.get()
                                    on:input=move |ev| set_new_description.set(event_target_value(&ev))
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                    placeholder="Enter description (optional)"
                                    rows="3"
                                />
                            </div>

                            // Due date and time
                            <div class="grid grid-cols-2 gap-4 mb-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        "Due Date"
                                    </label>
                                    <input
                                        type="date"
                                        prop:value=move || new_due_date.get()
                                        on:input=move |ev| set_new_due_date.set(event_target_value(&ev))
                                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        "Due Time"
                                    </label>
                                    <input
                                        type="time"
                                        prop:value=move || new_due_time.get()
                                        on:input=move |ev| set_new_due_time.set(event_target_value(&ev))
                                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                    />
                                </div>
                            </div>

                            // Assignee dropdown
                            <div class="mb-4">
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Assignee"
                                </label>
                                <select
                                    prop:value=move || new_assignee.get()
                                    on:change=move |ev| set_new_assignee.set(event_target_value(&ev))
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                >
                                    <option value="Mikko">"Mikko"</option>
                                    <option value="Niina">"Niina"</option>
                                </select>
                            </div>

                            // Status dropdown
                            <div class="mb-6">
                                <label class="block text-sm font-medium text-gray-700 mb-2">
                                    "Status"
                                </label>
                                <select
                                    prop:value=move || new_status.get()
                                    on:change=move |ev| set_new_status.set(event_target_value(&ev))
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                >
                                    <option value="Not Started">"Not Started"</option>
                                    <option value="In Progress">"In Progress"</option>
                                    <option value="Completed">"Completed"</option>
                                    <option value="Blocked">"Blocked"</option>
                                </select>
                            </div>

                            // Action buttons
                            <div class="flex gap-3">
                                <button
                                    type="button"
                                    on:click=move |_| {
                                        reset_form();
                                        set_show_modal.set(false);
                                    }
                                    class="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
                                    disabled=move || is_creating() || is_updating()
                                >
                                    "Cancel"
                                </button>
                                <button
                                    type="submit"
                                    class="flex-1 px-4 py-2 bg-gradient-to-r from-purple-500 to-fuchsia-500 text-white rounded-lg hover:from-purple-600 hover:to-fuchsia-600 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed"
                                    disabled=move || is_creating() || is_updating()
                                >
                                    <Show
                                        when=move || is_creating() || is_updating()
                                        fallback=move || if editing_todo.get().is_some() { "Update Todo" } else { "Create Todo" }
                                    >
                                        {move || if editing_todo.get().is_some() { "Updating..." } else { "Creating..." }}
                                    </Show>
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>
        </main>
    }
}
