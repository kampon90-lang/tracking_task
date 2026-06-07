#![allow(non_snake_case)]

use chrono::{Datelike, Local};
use dioxus::prelude::*;
mod components;
mod database;

// 💡 นำเข้า RecurringList
use crate::components::{AddTask, Calendar, RecurringList, TaskList};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Calendar,
    List,
    Recurring, // 💡 เพิ่ม Tab ใหม่
    AddTask,
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    if Local::now().day() == 16 {
        if let Ok(conn) = database::init_db() {
            let _ = database::run_maintenance(&conn);
            println!("🧹 Database maintenance (VACUUM & ANALYZE) completed!");
        }
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut current_tab = use_signal(|| Tab::List);

    let calendar_class = if current_tab() == Tab::Calendar {
        "bg-blue-600 text-white"
    } else {
        "bg-slate-800"
    };
    let list_class = if current_tab() == Tab::List {
        "bg-blue-600 text-white"
    } else {
        "bg-slate-800"
    };
    let recur_class = if current_tab() == Tab::Recurring {
        "bg-blue-600 text-white"
    } else {
        "bg-slate-800"
    };
    let add_class = if current_tab() == Tab::AddTask {
        "bg-blue-600 text-white"
    } else {
        "bg-slate-800"
    };

    rsx! {
        document::Link {rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "min-h-screen bg-slate-950 text-slate-300 p-4 sm:p-6",
            div { class: "max-w-xl mx-auto glass-card",
                h1 { class: "text-2xl font-black text-white text-center mb-8", "Task Tracker" }

                // 💡 เปลี่ยนเป็น 4 ปุ่ม (grid-cols-4)
                div { class: "grid grid-cols-4 gap-2 mb-6",
                    button { class: "py-2 text-[10px] sm:text-xs font-bold rounded-lg transition-all {calendar_class}", onclick: move |_| current_tab.set(Tab::Calendar), "📅 Calendar" }
                    button { class: "py-2 text-[10px] sm:text-xs font-bold rounded-lg transition-all {list_class}", onclick: move |_| current_tab.set(Tab::List), "📋 List" }
                    button { class: "py-2 text-[10px] sm:text-xs font-bold rounded-lg transition-all {recur_class}", onclick: move |_| current_tab.set(Tab::Recurring), "🔄 Recur" }
                    button { class: "py-2 text-[10px] sm:text-xs font-bold rounded-lg transition-all {add_class}", onclick: move |_| current_tab.set(Tab::AddTask), "➕ Add" }
                }

                match current_tab() {
                    Tab::Calendar => rsx! { Calendar {} },
                    Tab::List => rsx! { TaskList {} },
                    Tab::Recurring => rsx! { RecurringList {} }, // 💡 เรียกใช้หน้าใหม่
                    Tab::AddTask => rsx! { AddTask {} }
                }
            }
        }
    }
}
