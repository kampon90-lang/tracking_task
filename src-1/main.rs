#![allow(non_snake_case)]

use dioxus::prelude::*;

mod components;
mod database;

// 💡 แก้ไข: ดึง TaskList มารวมกับ AddTask
use crate::components::{AddTask, Calendar, TaskList};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Calendar,
    List,
    AddTask,
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
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
    let add_class = if current_tab() == Tab::AddTask {
        "bg-blue-600 text-white"
    } else {
        "bg-slate-800"
    };

    rsx! {
        document::Link {rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "min-h-screen bg-slate-950 text-slate-300 p-6",
            div { class: "max-w-xl mx-auto glass-card",
                h1 { class: "text-2xl font-black text-white text-center mb-8", "Task Tracker" }

                div { class: "flex gap-2 mb-6",
                    button { class: "flex-1 py-2 text-xs font-bold rounded-lg transition-all {calendar_class}", onclick: move |_| current_tab.set(Tab::Calendar), "📅 Calendar" }
                    button { class: "flex-1 py-2 text-xs font-bold rounded-lg transition-all {list_class}", onclick: move |_| current_tab.set(Tab::List), "📋 Task List" }
                    button { class: "flex-1 py-2 text-xs font-bold rounded-lg transition-all {add_class}", onclick: move |_| current_tab.set(Tab::AddTask), "➕ Add Task" }
                }

                match current_tab() {
                    Tab::Calendar => rsx! { Calendar {}  },
                    Tab::List => rsx! { TaskList {} }, // เรียกใช้ TaskList ได้แล้ว
                    Tab::AddTask => rsx! { AddTask {} }
                }
            }
        }
    }
}
