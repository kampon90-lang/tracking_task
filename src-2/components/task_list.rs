use crate::database::{
    get_all_tasks, init_db, update_task_details, update_task_status, TaskInstance, TaskTemplate,
};
use chrono::Local;
use dioxus::prelude::*;

#[component]
pub fn TaskList() -> Element {
    let mut tasks = use_signal(|| Vec::<(TaskInstance, TaskTemplate)>::new());

    // State สำหรับจัดการโหมดแก้ไข
    let mut editing_id = use_signal(|| None::<i32>);
    let mut edit_title = use_signal(String::new);
    let mut edit_note = use_signal(String::new);
    let mut edit_date = use_signal(String::new);

    let mut load_tasks = move || {
        if let Ok(conn) = init_db() {
            if let Ok(data) = get_all_tasks(&conn) {
                tasks.set(data);
            }
        }
    };

    use_effect(move || {
        load_tasks();
    });

    rsx! {
        div { class: "flex flex-col gap-4 max-h-[70vh] overflow-y-auto pr-2 custom-scrollbar",
            if tasks.read().is_empty() {
                div { class: "text-center py-10 text-slate-500 font-bold tracking-widest",
                    "📭 NO TASKS FOUND"
                }
            } else {
                {
                    tasks.read().iter().map(|(instance, template)| {
                        let today = Local::now().format("%Y-%m-%d").to_string();
                        let is_finished = instance.status == "finished";
                        let is_overdue = !is_finished && instance.due_date < today;

                        // เช็คว่าการ์ดใบนี้อยู่ในโหมดแก้ไขหรือไม่
                        let is_editing = editing_id() == Some(instance.id);

                        let status_color = if is_finished { "text-emerald-400" }
                            else if is_overdue { "text-red-400" }
                            else { "text-blue-400" };

                        let border_color = if is_finished { "border-emerald-900/50" }
                            else if is_overdue { "border-red-900/50" }
                            else { "border-slate-800" };

                        let bg_color = if is_finished { "bg-slate-900/50 opacity-60" }
                            else if is_overdue { "bg-red-950/20" }
                            else { "bg-slate-900" };

                        let title_class = if is_finished { "text-slate-500 line-through decoration-slate-500" }
                            else if is_overdue { "text-red-100" }
                            else { "text-white" };

                        let instance_id = instance.id;
                        let template_id = template.id;

                        rsx! {
                            if is_editing {
                                // --- 💡 UI สำหรับโหมดแก้ไข ---
                                div { key: "{instance.id}", class: "p-5 rounded-2xl border shadow-lg flex flex-col gap-3 bg-slate-800 border-blue-500",
                                    input {
                                        class: "w-full px-3 py-2 bg-slate-950 border border-slate-700 rounded-lg text-white font-bold focus:outline-none focus:ring-2 focus:ring-blue-500 shadow-inner",
                                        value: "{edit_title}",
                                        placeholder: "Work Name",
                                        oninput: move |e| edit_title.set(e.value())
                                    }
                                    textarea {
                                        class: "w-full px-3 py-2 bg-slate-950 border border-slate-700 rounded-lg text-slate-300 text-xs focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none h-16 shadow-inner",
                                        value: "{edit_note}",
                                        placeholder: "Note",
                                        oninput: move |e| edit_note.set(e.value())
                                    }
                                    div { class: "flex justify-between items-center mt-1",
                                        input {
                                            class: "w-36 px-2 py-1.5 bg-slate-950 text-white border border-slate-700 rounded-lg text-xs font-bold [color-scheme:dark] focus:outline-none focus:ring-2 focus:ring-blue-500 cursor-pointer shadow-inner",
                                            r#type: "date",
                                            value: "{edit_date}",
                                            oninput: move |e| edit_date.set(e.value())
                                        }
                                        div { class: "flex gap-2",
                                            button {
                                                class: "px-4 py-1.5 bg-slate-700 text-slate-300 hover:bg-slate-600 rounded-xl text-xs font-bold transition-all active:scale-95",
                                                onclick: move |_| editing_id.set(None),
                                                "Cancel"
                                            }
                                            button {
                                                class: "px-4 py-1.5 bg-blue-600 text-white hover:bg-blue-500 rounded-xl text-xs font-bold transition-all active:scale-95 shadow-md",
                                                onclick: move |_| {
                                                    if !edit_title().is_empty() && !edit_date().is_empty() {
                                                        if let Ok(conn) = init_db() {
                                                            let _ = update_task_details(&conn, instance_id, template_id, &edit_title(), &edit_note(), &edit_date());
                                                            load_tasks();
                                                            editing_id.set(None);
                                                        }
                                                    }
                                                },
                                                "💾 Save"
                                            }
                                        }
                                    }
                                }
                            } else {
                                // --- UI การ์ดแสดงผลปกติ ---
                                div { key: "{instance.id}", class: "p-5 rounded-2xl border shadow-lg flex flex-col gap-3 transition-all {border_color} {bg_color}",
                                    div { class: "flex justify-between items-start",
                                        div { class: "flex-1",
                                            h3 { class: "font-black text-lg {title_class}", "{template.title}" }

                                            if !template.note.is_empty() {
                                                p { class: "text-xs text-slate-400 mt-1 line-clamp-2", "{template.note}" }
                                            }
                                        }
                                        if template.recurrence != "none" {
                                            span { class: "px-2 py-1 bg-slate-800 text-slate-300 rounded-lg text-[10px] font-bold tracking-wider uppercase border border-slate-700", "🔄 {template.recurrence}" }
                                        }
                                    }
                                    div { class: "flex justify-between items-end mt-2 pt-3 border-t border-slate-800/50",
                                        div { class: "flex flex-col gap-1",
                                            span { class: "text-[10px] font-bold tracking-widest uppercase text-slate-500",
                                                if is_overdue { "🚨 OVERDUE" } else { "Due Date" }
                                            }
                                            span { class: "text-sm font-bold {status_color}", "📅 {instance.due_date}" }
                                        }
                                        div { class: "flex gap-2",
                                            if !is_finished {
                                                // --- ปุ่ม Edit ---
                                                button {
                                                    class: "px-3 py-1.5 bg-orange-900/20 text-orange-400 hover:bg-orange-800/40 hover:text-orange-200 border border-orange-800/50 rounded-xl text-xs font-bold transition-all active:scale-95",
                                                    onclick: {
                                                        // โคลนค่าปัจจุบันไปใส่ใน State แก้ไขก่อนเปิดฟอร์ม
                                                        let current_title = template.title.clone();
                                                        let current_note = template.note.clone();
                                                        let current_date = instance.due_date.clone();
                                                        move |_| {
                                                            edit_title.set(current_title.clone());
                                                            edit_note.set(current_note.clone());
                                                            edit_date.set(current_date.clone());
                                                            editing_id.set(Some(instance_id));
                                                        }
                                                    },
                                                    "✏️ Edit"
                                                }

                                                // --- ปุ่ม Finish ---
                                                button {
                                                    class: "px-3 py-1.5 bg-emerald-900/30 text-emerald-400 hover:bg-emerald-800/50 hover:text-white border border-emerald-800 rounded-xl text-xs font-bold transition-all active:scale-95",
                                                    onclick: move |_| {
                                                        if let Ok(conn) = init_db() {
                                                            let _ = update_task_status(&conn, instance_id, "finished");
                                                            load_tasks();
                                                        }
                                                    },
                                                    "✔️ Finish"
                                                }
                                            } else {
                                                // --- ปุ่ม Undo ---
                                                button {
                                                    class: "px-3 py-1.5 bg-slate-800 text-slate-400 hover:bg-slate-700 hover:text-white border border-slate-700 rounded-xl text-xs font-bold transition-all active:scale-95",
                                                    onclick: move |_| {
                                                        if let Ok(conn) = init_db() {
                                                            let _ = update_task_status(&conn, instance_id, "ongoing");
                                                            load_tasks();
                                                        }
                                                    },
                                                    "↩️ Undo"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    })
                }
            }
        }
    }
}
