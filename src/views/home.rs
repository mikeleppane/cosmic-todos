use std::str::FromStr;

use crate::app::{create_todo_server, delete_todo_server, get_todos_server, update_todo_server};
use crate::todo::{Todo, TodoAssignee, TodoStatus};
use crate::views::status_bar::StatusBar;
use chrono::{Datelike, Local, NaiveDate, TimeZone};
use leptos::leptos_dom::logging;
use leptos::web_sys;
use leptos::{ev, prelude::*};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Title,
    DueDate,
    Status,
    Assignee,
    CreatedDate,
}

impl SortBy {
    fn as_str(&self) -> &'static str {
        match self {
            SortBy::Title => "title",
            SortBy::DueDate => "due_date",
            SortBy::Status => "status",
            SortBy::Assignee => "assignee",
            SortBy::CreatedDate => "created_date",
        }
    }
}

impl std::str::FromStr for SortBy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "title" => Ok(SortBy::Title),
            "due_date" => Ok(SortBy::DueDate),
            "status" => Ok(SortBy::Status),
            "assignee" => Ok(SortBy::Assignee),
            "created_date" => Ok(SortBy::CreatedDate),
            _ => Err(format!("Unknown sort type: {s}")),
        }
    }
}

#[component]
#[allow(clippy::must_use_candidate)]
#[allow(clippy::too_many_lines)]
pub fn SearchAndFilters(
    search_term: ReadSignal<String>,
    set_search_term: WriteSignal<String>,
    filter_status: ReadSignal<String>,
    set_filter_status: WriteSignal<String>,
    filter_assignee: ReadSignal<String>,
    set_filter_assignee: WriteSignal<String>,
    sort_by: ReadSignal<SortBy>,
    set_sort_by: WriteSignal<SortBy>,
    sort_ascending: ReadSignal<bool>,
    set_sort_ascending: WriteSignal<bool>,
    total_todos: impl Fn() -> usize + Send + 'static,
    filtered_todos: impl Fn() -> usize + Send + 'static,
) -> impl IntoView {
    let clear_filters = move |_| {
        set_search_term.set(String::new());
        set_filter_status.set("All".to_string());
        set_filter_assignee.set("All".to_string());
    };

    view! {
        <div class="bg-white rounded-xl shadow-sm border border-gray-100 p-4 mb-4">
            // Search bar
            <div class="mb-4">
                <label class="block text-sm font-medium text-gray-700 mb-2">"Search todos"</label>
                <div class="relative">
                    <input
                        type="text"
                        prop:value=move || search_term.get()
                        on:input=move |ev| set_search_term.set(event_target_value(&ev))
                        class="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                        placeholder="Search by title or description..."
                    />
                    <svg
                        class="absolute left-3 top-2.5 h-5 w-5 text-gray-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                        />
                    </svg>
                </div>
            </div>

            // Filters and sorting row
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                // Status filter
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Status"</label>
                    <select
                        prop:value=move || filter_status.get()
                        on:change=move |ev| set_filter_status.set(event_target_value(&ev))
                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent text-sm"
                    >
                        <option value="All">"All Status"</option>
                        <option value="Pending">"Pending"</option>
                        <option value="Completed">"Completed"</option>
                    </select>
                </div>

                // Assignee filter
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Assignee"</label>
                    <select
                        prop:value=move || filter_assignee.get()
                        on:change=move |ev| set_filter_assignee.set(event_target_value(&ev))
                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent text-sm"
                    >
                        <option value="All">"All Assignees"</option>
                        <option value="Mikko">"Mikko"</option>
                        <option value="Niina">"Niina"</option>
                    </select>
                </div>

                // Sort by
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Sort by"</label>
                    <select
                        prop:value=move || sort_by.get().as_str()
                        on:change=move |ev| {
                            set_sort_by
                                .set(
                                    SortBy::from_str(&event_target_value(&ev))
                                        .map_err(|e| logging::console_warn(
                                            &format!("Invalid sort option: {e}"),
                                        ))
                                        .unwrap_or(SortBy::CreatedDate),
                                );
                        }
                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent text-sm"
                    >
                        <option value="created_date">"Created Date"</option>
                        <option value="title">"Title"</option>
                        <option value="due_date">"Due Date"</option>
                        <option value="status">"Status"</option>
                        <option value="assignee">"Assignee"</option>
                    </select>
                </div>

                // Sort order toggle
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Order"</label>
                    <button
                        on:click=move |_| set_sort_ascending.update(|asc| *asc = !*asc)
                        class="w-full px-3 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors text-sm flex items-center justify-center gap-2"
                    >
                        {move || if sort_ascending.get() { "Ascending" } else { "Descending" }}
                        <svg
                            class=move || {
                                format!(
                                    "w-4 h-4 transition-transform {}",
                                    if sort_ascending.get() { "rotate-0" } else { "rotate-180" },
                                )
                            }
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M5 15l7-7 7 7"
                            />
                        </svg>
                    </button>
                </div>
            </div>

            // Results count and clear filters
            <div class="mt-3 pt-3 border-t border-gray-100 flex justify-between items-center">
                <p class="text-sm text-gray-600">
                    {move || format!("Showing {} of {} todos", filtered_todos(), total_todos())}
                </p>

                <Show when=move || {
                    !search_term.get().is_empty() || filter_status.get() != "All"
                        || filter_assignee.get() != "All"
                }>
                    <button
                        on:click=clear_filters
                        class="px-3 py-1 text-sm text-purple-600 border border-purple-200 rounded-lg hover:bg-purple-50 transition-colors"
                    >
                        "Clear Filters"
                    </button>
                </Show>
            </div>
        </div>
    }
}

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
    let (new_status, set_new_status) = signal("Pending".to_string());

    // Sorting and filtering state
    let (sort_by, set_sort_by) = signal(SortBy::CreatedDate);
    let (sort_ascending, set_sort_ascending) = signal(false);
    let (filter_status, set_filter_status) = signal("All".to_string());
    let (filter_assignee, set_filter_assignee) = signal("All".to_string());
    let (search_term, set_search_term) = signal(String::new());

    // Helper to reset form
    let reset_form = move || {
        set_new_title.set(String::new());
        set_new_description.set(String::new());
        set_new_due_date.set(String::new());
        set_new_due_time.set(String::new());
        set_new_assignee.set("Mikko".to_string());
        set_new_status.set("Pending".to_string());
        set_editing_todo.set(None);
    };

    // Helper to populate form with existing todo data
    let populate_form = move |todo: &Todo| {
        set_new_title.set(todo.title.clone());
        set_new_description.set(todo.description.clone().unwrap_or_default());
        set_new_assignee.set(todo.assignee.as_str().to_string());
        set_new_status.set(todo.status.as_str().to_string());

        if let Some(timestamp) = todo.due_date {
            if let Ok(timestamp_i64) = i64::try_from(timestamp) {
                if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp_i64, 0) {
                    let local_datetime = datetime.with_timezone(&chrono::Local);
                    set_new_due_date.set(local_datetime.format("%Y-%m-%d").to_string());
                    set_new_due_time.set(local_datetime.format("%H:%M").to_string());
                }
            }
        } else {
            set_new_due_date.set(String::new());
            set_new_due_time.set(String::new());
        }
    };

    // Filter and sort todos
    let filtered_and_sorted_todos = move || {
        let mut todos_list = todos.get();
        let search = search_term.get().to_lowercase();
        let status_filter = filter_status.get();
        let assignee_filter = filter_assignee.get();

        // Apply filters
        todos_list.retain(|todo| {
            // Search filter
            let matches_search = search.is_empty()
                || todo.title.to_lowercase().contains(&search)
                || todo
                    .description
                    .as_ref()
                    .is_some_and(|desc| desc.to_lowercase().contains(&search));

            // Status filter
            let matches_status = status_filter == "All" || todo.status.as_str() == status_filter;

            // Assignee filter
            let matches_assignee =
                assignee_filter == "All" || todo.assignee.as_str() == assignee_filter;

            matches_search && matches_status && matches_assignee
        });

        // Apply sorting
        let sort_criteria = move || sort_by.get();
        let ascending = move || sort_ascending.get();

        todos_list.sort_by(|a, b| {
            let comparison = match sort_criteria() {
                SortBy::Title => a.title.cmp(&b.title),
                SortBy::DueDate => match (a.due_date, b.due_date) {
                    (Some(a_date), Some(b_date)) => a_date.cmp(&b_date),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                },
                SortBy::Status => a.status.as_str().cmp(b.status.as_str()),
                SortBy::Assignee => a.assignee.as_str().cmp(b.assignee.as_str()),
                SortBy::CreatedDate => a.id.cmp(&b.id),
            };

            if ascending() {
                comparison
            } else {
                comparison.reverse()
            }
        });

        todos_list
    };

    let grouped_todos = move || {
        use std::collections::BTreeMap;

        let todos_list = filtered_and_sorted_todos();
        let mut groups: BTreeMap<String, Vec<Todo>> = BTreeMap::new();

        for todo in todos_list {
            let group_key = if let Some(due_timestamp) = todo.due_date {
                if let Ok(timestamp_i64) = i64::try_from(due_timestamp) {
                    if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp_i64, 0) {
                        let local_datetime = datetime.with_timezone(&chrono::Local);
                        local_datetime.format("%Y-%m").to_string()
                    } else {
                        "Invalid Date".to_string()
                    }
                } else {
                    "No Due Date".to_string()
                }
            } else {
                "No Due Date".to_string()
            };

            groups.entry(group_key).or_default().push(todo);
        }

        // Sort todos within each group by due date
        for todos in groups.values_mut() {
            todos.sort_by(|a, b| {
                match (a.due_date, b.due_date) {
                    (Some(a_date), Some(b_date)) => a_date.cmp(&b_date),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.title.cmp(&b.title), // Fallback to title if no dates
                }
            });
        }

        groups
    };

    let format_month_header = |month_key: &str| -> String {
        if month_key == "No Due Date" {
            "No Due Date".to_string()
        } else if let Ok(date) =
            chrono::NaiveDate::parse_from_str(&format!("{month_key}-01"), "%Y-%m-%d")
        {
            date.format("%B %Y").to_string()
        } else {
            month_key.to_string()
        }
    };

    // Calendar helper functions
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

    let is_overdue = |due_timestamp: u64| -> bool {
        if let Ok(timestamp_i64) = i64::try_from(due_timestamp) {
            if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp_i64, 0) {
                let due_date = datetime.with_timezone(&chrono::Local);
                let now = chrono::Local::now();
                due_date < now
            } else {
                false
            }
        } else {
            false
        }
    };

    let is_past_date = move || {
        let date_str = new_due_date.get();
        let time_str = new_due_time.get();

        if date_str.is_empty() {
            return false;
        }

        let time_str = if time_str.is_empty() {
            "00:00"
        } else {
            &time_str
        };
        let datetime_str = format!("{date_str} {time_str}");

        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M") {
            if let Some(local_dt) = chrono::Local.from_local_datetime(&dt).single() {
                local_dt < chrono::Local::now()
            } else {
                false
            }
        } else {
            false
        }
    };

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
                        if let Some(todo) = todos.iter_mut().find(|t| t.id == updated_todo.id) {
                            *todo = updated_todo;
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
                Ok(()) => {
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
                if let Some(local_dt) = chrono::Local.from_local_datetime(&dt).single() {
                    let timestamp_i64 = local_dt.timestamp();
                    let Ok(timestamp) = u64::try_from(timestamp_i64) else {
                        set_error_message.set("Due date cannot be before 1970".to_string());
                        return;
                    };

                    // Check if the due date is in the past and show warning
                    let now = chrono::Local::now();
                    if local_dt < now {
                        // Only show warning for new todos, not when editing existing ones
                        if editing_todo.get_untracked().is_none() {
                            let time_diff = now.signed_duration_since(local_dt);
                            let warning_msg = if time_diff.num_days() > 0 {
                                format!(
                                    "Warning: You're creating a todo with a due date {} day(s) in the past. Are you sure you want to continue?",
                                    time_diff.num_days()
                                )
                            } else if time_diff.num_hours() > 0 {
                                format!(
                                    "Warning: You're creating a todo with a due date {} hour(s) in the past. Are you sure you want to continue?",
                                    time_diff.num_hours()
                                )
                            } else {
                                "Warning: You're creating a todo with a due date in the past. Are you sure you want to continue?".to_string()
                            };

                            // Show confirmation dialog
                            if let Some(window) = web_sys::window() {
                                if !window.confirm_with_message(&warning_msg).unwrap_or(false) {
                                    return; // User cancelled, don't create the todo
                                }
                            }
                        }
                    }

                    Some(timestamp)
                } else {
                    set_error_message.set("Invalid local datetime".to_string());
                    return;
                }
            } else {
                set_error_message.set("Invalid date/time format".to_string());
                return;
            }
        };

        let todo = Todo {
            id: editing_todo.get_untracked().map_or_else(
                || match Uuid::new_v4().to_string() {
                    id if !id.is_empty() => id,
                    _ => {
                        logging::console_error("Failed to generate UUID");
                        format!("fallback-{}", chrono::Utc::now().timestamp())
                    }
                },
                |t| t.id,
            ),
            title: title.trim().to_string(),
            description: if new_description.get_untracked().trim().is_empty() {
                None
            } else {
                Some(new_description.get_untracked().trim().to_string())
            },
            due_date: due_timestamp,
            assignee: TodoAssignee::from_str(&new_assignee.get_untracked())
                .map_err(|e| leptos::logging::warn!("valid assignee: {:#?}", e))
                .unwrap_or(TodoAssignee::Mikko),
            status: TodoStatus::from_str(&new_status.get_untracked())
                .map_err(|e| leptos::logging::warn!("Invalid status: {:#?}", e))
                .unwrap_or(TodoStatus::Pending),
        };

        match todo.validate() {
            Ok(()) => {}
            Err(e) => {
                set_error_message.set(format!("Invalid todo data: Error validating todo: {e}"));
                return;
            }
        }

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

    let format_due_date = |timestamp: u64| -> String {
        if let Ok(timestamp_i64) = i64::try_from(timestamp) {
            if let Some(datetime) = chrono::DateTime::from_timestamp(timestamp_i64, 0) {
                datetime.format("%A, %B %d, %Y at %I:%M %p").to_string()
            } else {
                "Invalid date".to_string()
            }
        } else {
            "Invalid date".to_string()
        }
    };

    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <div class="min-h-screen flex items-center justify-center">
                    <div class="text-center p-8 bg-red-50 rounded-lg border border-red-200">
                        <h2 class="text-xl font-bold text-red-800 mb-4">"Something went wrong"</h2>
                        <details class="text-left">
                            <summary class="cursor-pointer text-red-600 mb-2">
                                "Error details"
                            </summary>
                            <pre class="text-sm text-red-700 whitespace-pre-wrap">
                                {format!("{:#?}", errors.get())}
                            </pre>
                        </details>
                    </div>
                </div>
            }
        }>
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
                    <img
                        src="/images/familyleppanen-logo.png"
                        alt="Family Todos Logo"
                        class="h-10 w-auto"
                        style="width: 50px; height: 50px;"
                    />
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
                    // Calendar section
                    <div class="lg:col-span-1">
                        <div class="bg-white rounded-2xl shadow-sm border border-gray-100 p-6">
                            <div class="flex justify-between items-center mb-4">
                                <h2 class="text-xl font-semibold text-gray-800">
                                    {move || {
                                        format!(
                                            "{} {}",
                                            get_month_name(current_month.get()),
                                            current_year.get(),
                                        )
                                    }}
                                </h2>
                                <div class="flex gap-2">
                                    <button
                                        on:click=prev_month
                                        class="p-2 hover:bg-gray-100 rounded-lg transition-colors"
                                    >
                                        <svg
                                            class="w-4 h-4"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M15 19l-7-7 7-7"
                                            />
                                        </svg>
                                    </button>
                                    <button
                                        on:click=next_month
                                        class="p-2 hover:bg-gray-100 rounded-lg transition-colors"
                                    >
                                        <svg
                                            class="w-4 h-4"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M9 5l7 7-7 7"
                                            />
                                        </svg>
                                    </button>
                                </div>
                            </div>

                            // Calendar grid
                            <div class="grid grid-cols-7 gap-1 mb-2">
                                {
                                    const DAYS: &[&str] = &[
                                        "Sun",
                                        "Mon",
                                        "Tue",
                                        "Wed",
                                        "Thu",
                                        "Fri",
                                        "Sat",
                                    ];
                                    DAYS.iter()
                                        .map(|day| {
                                            view! {
                                                <div class="p-2 text-center text-xs font-medium text-gray-500">
                                                    {*day}
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>()
                                }
                            </div>

                            <div class="grid grid-cols-7 gap-1">
                                {move || {
                                    let year = current_year.get();
                                    let month = current_month.get();
                                    let days_in_month = get_days_in_month(year, month);
                                    let first_day = get_first_day_of_month(year, month);
                                    let mut calendar_days = Vec::new();
                                    for _ in 0..first_day {
                                        calendar_days
                                            .push(

                                                view! { <div class="p-2 h-8">{String::new()}</div> },
                                            );
                                    }
                                    for day in 1..=days_in_month {
                                        let is_today = if let Some(current_date) = NaiveDate::from_ymd_opt(
                                            year,
                                            month,
                                            day,
                                        ) {
                                            current_date == today
                                        } else {
                                            false
                                        };
                                        if is_today {
                                            calendar_days
                                                .push(

                                                    view! {
                                                        <div class="p-2 h-8 text-center text-sm rounded-lg bg-gradient-to-r from-purple-500 to-fuchsia-500 text-white font-semibold">
                                                            {format!("{day}")}
                                                        </div>
                                                    },
                                                );
                                        } else {
                                            calendar_days
                                                .push(
                                                    view! {
                                                        <div class="p-2 h-8 text-center text-sm rounded-lg hover:bg-gray-100 cursor-pointer transition-colors">
                                                            {format!("{day}")}
                                                        </div>
                                                    },
                                                );
                                        }
                                    }
                                    calendar_days
                                }}
                            </div>

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

                    // Todo list section
                    <div class="lg:col-span-2">
                        // Search and filter controls
                        <SearchAndFilters
                            search_term=search_term
                            set_search_term=set_search_term
                            filter_status=filter_status
                            set_filter_status=set_filter_status
                            filter_assignee=filter_assignee
                            set_filter_assignee=set_filter_assignee
                            sort_by=sort_by
                            set_sort_by=set_sort_by
                            sort_ascending=sort_ascending
                            set_sort_ascending=set_sort_ascending
                            total_todos=move || todos.get().len()
                            filtered_todos=move || filtered_and_sorted_todos().len()
                        />

                        <Show when=move || loading.get()>
                            <div class="flex justify-center items-center py-8">
                                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-600"></div>
                                <span class="ml-2 text-gray-600">"Loading todos..."</span>
                            </div>
                        </Show>

                        <Show when=move || !loading.get()>
                            <div class="space-y-6">
                                {move || {
                                    let todos_groups = grouped_todos();
                                    if todos_groups.is_empty() {
                                        let has_filters = !search_term.get().is_empty()
                                            || filter_status.get() != "All"
                                            || filter_assignee.get() != "All";
                                        if has_filters {

                                            view! {
                                                <div class="text-center py-12 bg-white rounded-2xl shadow-sm border border-gray-100">
                                                    <div class="text-gray-400 mb-4">
                                                        <svg
                                                            class="mx-auto h-12 w-12"
                                                            fill="none"
                                                            stroke="currentColor"
                                                            viewBox="0 0 24 24"
                                                        >
                                                            <path
                                                                stroke-linecap="round"
                                                                stroke-linejoin="round"
                                                                stroke-width="2"
                                                                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                                                            />
                                                        </svg>
                                                    </div>
                                                    <h3 class="text-lg font-medium text-gray-900 mb-2">
                                                        "No todos match your filters"
                                                    </h3>
                                                    <p class="text-gray-500 mb-4">
                                                        "Try adjusting your search or filter criteria."
                                                    </p>
                                                    <button
                                                        on:click=move |_| {
                                                            set_search_term.set(String::new());
                                                            set_filter_status.set("All".to_string());
                                                            set_filter_assignee.set("All".to_string());
                                                        }
                                                        class="px-4 py-2 text-purple-600 border border-purple-200 rounded-lg hover:bg-purple-50 transition-colors"
                                                    >
                                                        "Clear Filters"
                                                    </button>
                                                </div>
                                            }
                                                .into_any()
                                        } else {
                                            view! {
                                                <div class="text-center py-12 bg-white rounded-2xl shadow-sm border border-gray-100">
                                                    <div class="text-gray-400 mb-4">
                                                        <svg
                                                            class="mx-auto h-12 w-12"
                                                            fill="none"
                                                            stroke="currentColor"
                                                            viewBox="0 0 24 24"
                                                        >
                                                            <path
                                                                stroke-linecap="round"
                                                                stroke-linejoin="round"
                                                                stroke-width="2"
                                                                d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
                                                            />
                                                        </svg>
                                                    </div>
                                                    <h3 class="text-lg font-medium text-gray-900 mb-2">
                                                        "No todos yet"
                                                    </h3>
                                                    <p class="text-gray-500 mb-4">
                                                        "Create your first todo to get started!"
                                                    </p>
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
                                            }
                                                .into_any()
                                        }
                                    } else {
                                        view! {
                                            <div class="space-y-6">
                                                {todos_groups
                                                    .into_iter()
                                                    .map(|(month_key, todos_in_month)| {
                                                        let month_header = format_month_header(&month_key);
                                                        view! {
                                                            <div class="space-y-4">
                                                                // Month header
                                                                <div class="flex items-center gap-4">
                                                                    <h3 class="text-xl font-semibold text-gray-800">
                                                                        {month_header}
                                                                    </h3>
                                                                    <div class="flex-1 h-px bg-gradient-to-r from-purple-200 to-transparent"></div>
                                                                    <span class="text-sm text-gray-500 bg-gray-100 px-2 py-1 rounded-full">
                                                                        {format!("{} todos", todos_in_month.len())}
                                                                    </span>
                                                                </div>

                                                                // Todos in this month
                                                                <div class="grid gap-4">
                                                                    {todos_in_month
                                                                        .into_iter()
                                                                        .map(|todo| {
                                                                            let todo_clone = todo.clone();
                                                                            let todo_id = todo.id;
                                                                            let status_color = match todo.status {
                                                                                TodoStatus::Pending => "bg-gray-100 text-gray-800",
                                                                                TodoStatus::Completed => "bg-green-100 text-green-800",
                                                                            };
                                                                            let assignee_color = match todo.assignee {
                                                                                TodoAssignee::Mikko => "bg-purple-100 text-purple-800",
                                                                                TodoAssignee::Niina => "bg-pink-100 text-pink-800",
                                                                            };
                                                                            let is_todo_overdue = todo
                                                                                .due_date
                                                                                .is_some_and(|timestamp| {
                                                                                    is_overdue(timestamp) && todo.status == TodoStatus::Pending
                                                                                });
                                                                            let card_classes = if is_todo_overdue {
                                                                                "bg-red-50 border-red-200 rounded-xl shadow-sm border p-6 hover:shadow-md transition-shadow duration-200"
                                                                            } else {
                                                                                "bg-white rounded-xl shadow-sm border border-gray-100 p-6 hover:shadow-md transition-shadow duration-200"
                                                                            };

                                                                            // Check if todo is overdue and not completed

                                                                            // Apply overdue styling

                                                                            view! {
                                                                                <div class=card_classes>
                                                                                    <div class="flex justify-between items-start mb-3">
                                                                                        <div class="flex items-start gap-2">
                                                                                            // Add overdue indicator icon
                                                                                            {if is_todo_overdue {
                                                                                                view! {
                                                                                                    <svg
                                                                                                        class="w-5 h-5 text-red-500 mt-0.5 flex-shrink-0"
                                                                                                        fill="currentColor"
                                                                                                        viewBox="0 0 20 20"
                                                                                                    >
                                                                                                        <path
                                                                                                            fill-rule="evenodd"
                                                                                                            d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                                                                                                            clip-rule="evenodd"
                                                                                                        />
                                                                                                    </svg>
                                                                                                }
                                                                                                    .into_any()
                                                                                            } else {
                                                                                                view! { <div></div> }.into_any()
                                                                                            }}
                                                                                            <h4 class=format!(
                                                                                                "text-lg font-semibold {}",
                                                                                                if is_todo_overdue {
                                                                                                    "text-red-900"
                                                                                                } else {
                                                                                                    "text-gray-900"
                                                                                                },
                                                                                            )>{todo.title.clone()}</h4>
                                                                                        </div>
                                                                                        <div class="flex items-center gap-2">
                                                                                            <span class=format!(
                                                                                                "px-2 py-1 text-xs font-medium rounded-full {status_color}",
                                                                                            )>{todo.status.as_str()}</span>
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
                                                                                                    <svg
                                                                                                        class="w-4 h-4"
                                                                                                        fill="none"
                                                                                                        stroke="currentColor"
                                                                                                        viewBox="0 0 24 24"
                                                                                                    >
                                                                                                        <path
                                                                                                            stroke-linecap="round"
                                                                                                            stroke-linejoin="round"
                                                                                                            stroke-width="2"
                                                                                                            d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                                                                                                        />
                                                                                                    </svg>
                                                                                                </button>
                                                                                                <button
                                                                                                    on:click=move |_| {
                                                                                                        if let Some(window) = web_sys::window() {
                                                                                                            if window
                                                                                                                .confirm_with_message(
                                                                                                                    "Are you sure you want to delete this todo?",
                                                                                                                )
                                                                                                                .unwrap_or(false)
                                                                                                            {
                                                                                                                delete_todo_action.dispatch(todo_id.to_string());
                                                                                                            }
                                                                                                        }
                                                                                                    }
                                                                                                    class="p-1 text-gray-500 hover:text-red-600 hover:bg-red-50 rounded transition-colors"
                                                                                                    title="Delete todo"
                                                                                                    disabled=is_deleting
                                                                                                >
                                                                                                    <svg
                                                                                                        class="w-4 h-4"
                                                                                                        fill="none"
                                                                                                        stroke="currentColor"
                                                                                                        viewBox="0 0 24 24"
                                                                                                    >
                                                                                                        <path
                                                                                                            stroke-linecap="round"
                                                                                                            stroke-linejoin="round"
                                                                                                            stroke-width="2"
                                                                                                            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                                                                                                        />
                                                                                                    </svg>
                                                                                                </button>
                                                                                            </div>
                                                                                        </div>
                                                                                    </div>

                                                                                    {todo
                                                                                        .description
                                                                                        .as_ref()
                                                                                        .map(|desc| {
                                                                                            view! {
                                                                                                <p class=format!(
                                                                                                    "mb-3 {}",
                                                                                                    if is_todo_overdue {
                                                                                                        "text-red-700"
                                                                                                    } else {
                                                                                                        "text-gray-600"
                                                                                                    },
                                                                                                )>{desc.clone()}</p>
                                                                                            }
                                                                                        })}

                                                                                    <div class="flex flex-wrap gap-2 items-center">
                                                                                        <span class=format!(
                                                                                            "px-2 py-1 text-xs font-medium rounded-full {assignee_color}",
                                                                                        )>{todo.assignee.as_str()}</span>

                                                                                        {todo
                                                                                            .due_date
                                                                                            .map(|timestamp| {
                                                                                                let due_date_class = if is_overdue(timestamp)
                                                                                                    && todo.status == TodoStatus::Pending
                                                                                                {
                                                                                                    "px-2 py-1 text-xs font-medium rounded-full bg-red-200 text-red-900 font-bold"
                                                                                                } else {
                                                                                                    "px-2 py-1 text-xs font-medium rounded-full bg-yellow-100 text-yellow-800"
                                                                                                };

                                                                                                view! {
                                                                                                    <span class=due_date_class>
                                                                                                        {if is_overdue(timestamp)
                                                                                                            && todo.status == TodoStatus::Pending
                                                                                                        {
                                                                                                            format!("OVERDUE: {}", format_due_date(timestamp))
                                                                                                        } else {
                                                                                                            format!("Due: {}", format_due_date(timestamp))
                                                                                                        }}
                                                                                                    </span>
                                                                                                }
                                                                                            })}
                                                                                    </div>
                                                                                </div>
                                                                            }
                                                                        })
                                                                        .collect::<Vec<_>>()}
                                                                </div>
                                                            </div>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()}
                                            </div>
                                        }
                                            .into_any()
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
                                    {move || {
                                        if editing_todo.get().is_some() {
                                            "Edit Todo"
                                        } else {
                                            "Create New Todo"
                                        }
                                    }}
                                </h2>
                                <button
                                    on:click=move |_| set_show_modal.set(false)
                                    class="text-gray-500 hover:text-gray-700 text-2xl leading-none"
                                >
                                    "×"
                                </button>
                            </div>

                            <form on:submit=handle_submit>
                                <div class="mb-4">
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        "Title *"
                                    </label>
                                    <input
                                        type="text"
                                        prop:value=move || new_title.get()
                                        on:input=move |ev| {
                                            set_new_title.set(event_target_value(&ev));
                                        }
                                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                        placeholder="Enter todo title"
                                        required
                                    />
                                </div>

                                <div class="mb-4">
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        "Description"
                                    </label>
                                    <textarea
                                        prop:value=move || new_description.get()
                                        on:input=move |ev| {
                                            set_new_description.set(event_target_value(&ev));
                                        }
                                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                        placeholder="Enter description (optional)"
                                        rows="3"
                                    />
                                </div>

                                <div class="grid grid-cols-2 gap-4 mb-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-2">
                                            "Due Date"
                                        </label>
                                        <input
                                            type="date"
                                            prop:value=move || new_due_date.get()
                                            on:input=move |ev| {
                                                set_new_due_date.set(event_target_value(&ev));
                                            }
                                            class=move || {
                                                if is_past_date() {
                                                    "w-full px-3 py-2 border border-orange-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-transparent bg-orange-50"
                                                } else {
                                                    "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                                }
                                            }
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-2">
                                            "Due Time"
                                        </label>
                                        <input
                                            type="time"
                                            prop:value=move || new_due_time.get()
                                            on:input=move |ev| {
                                                set_new_due_time.set(event_target_value(&ev));
                                            }
                                            class=move || {
                                                if is_past_date() {
                                                    "w-full px-3 py-2 border border-orange-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-transparent bg-orange-50"
                                                } else {
                                                    "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                                }
                                            }
                                        />
                                    </div>
                                </div>
                                <Show when=move || is_past_date() && editing_todo.get().is_none()>
                                    <div class="mb-4 p-2 rounded-lg bg-orange-50 border border-orange-200">
                                        <div class="flex items-center gap-2">
                                            <svg
                                                class="w-4 h-4 text-orange-500 flex-shrink-0"
                                                fill="currentColor"
                                                viewBox="0 0 20 20"
                                            >
                                                <path
                                                    fill-rule="evenodd"
                                                    d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                                                    clip-rule="evenodd"
                                                />
                                            </svg>
                                            <p class="text-sm text-orange-700">
                                                "This date/time is in the past. You'll be asked to confirm when creating the todo."
                                            </p>
                                        </div>
                                    </div>
                                </Show>

                                <div class="mb-4">
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        "Assignee"
                                    </label>
                                    <select
                                        prop:value=move || new_assignee.get()
                                        on:change=move |ev| {
                                            set_new_assignee.set(event_target_value(&ev));
                                        }
                                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                    >
                                        <option value="Mikko">"Mikko"</option>
                                        <option value="Niina">"Niina"</option>
                                    </select>
                                </div>

                                <div class="mb-6">
                                    <label class="block text-sm font-medium text-gray-700 mb-2">
                                        "Status"
                                    </label>
                                    <select
                                        prop:value=move || new_status.get()
                                        on:change=move |ev| {
                                            set_new_status.set(event_target_value(&ev));
                                        }
                                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                                    >
                                        <option value="Pending">"Pending"</option>
                                        <option value="Completed">"Completed"</option>
                                    </select>
                                </div>

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
                                            fallback=move || {
                                                if editing_todo.get().is_some() {
                                                    "Update Todo"
                                                } else {
                                                    "Create Todo"
                                                }
                                            }
                                        >
                                            {move || {
                                                if editing_todo.get().is_some() {
                                                    "Updating..."
                                                } else {
                                                    "Creating..."
                                                }
                                            }}
                                        </Show>
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                </Show>
            </main>
            <StatusBar />
        </ErrorBoundary>
    }
}
