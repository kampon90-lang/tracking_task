use crate::database::{get_all_tasks, init_db, update_task_status, TaskInstance, TaskTemplate};
use dioxus::prelude::*;

#[component]
pub fn TaskList() -> Element {
    let mut tasks = use_signal(|| Vec::<(TaskInstance, TaskTemplate)>::new());

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
                        let is_finished = instance.status == "finished";
                        let status_color = if is_finished { "text-emerald-400" } else { "text-blue-400" };
                        let border_color = if is_finished { "border-emerald-900/50" } else { "border-slate-800" };
                        let bg_color = if is_finished { "bg-slate-900/50 opacity-60" } else { "bg-slate-900" };

                        // 💡 นำ Logic if-else ออกมาเป็นตัวแปรตรงนี้
                        let title_class = if is_finished {
                            "text-slate-500 line-through decoration-slate-500"
                        } else {
                            "text-white"
                        };

                        let instance_id_finish = instance.id;
                        let instance_id_undo = instance.id;

                        rsx! {
                            div { key: "{instance.id}", class: "p-5 rounded-2xl border shadow-lg flex flex-col gap-3 transition-all {border_color} {bg_color}",
                                div { class: "flex justify-between items-start",
                                    div { class: "flex-1",
                                        // 💡 เรียกใช้ตัวแปร title_class
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
                                        span { class: "text-[10px] font-bold tracking-widest uppercase text-slate-500", "Due Date" }
                                        span { class: "text-sm font-bold {status_color}", "📅 {instance.due_date}" }
                                    }
                                    div { class: "flex gap-2",
                                        if !is_finished {
                                            button {
                                                class: "px-3 py-1.5 bg-emerald-900/30 text-emerald-400 hover:bg-emerald-800/50 hover:text-white border border-emerald-800 rounded-xl text-xs font-bold transition-all active:scale-95",
                                                onclick: move |_| {
                                                    if let Ok(conn) = init_db() {
                                                        let _ = update_task_status(&conn, instance_id_finish, "finished");
                                                        load_tasks();
                                                    }
                                                },
                                                "✔️ Finish"
                                            }
                                        } else {
                                            button {
                                                class: "px-3 py-1.5 bg-slate-800 text-slate-400 hover:bg-slate-700 hover:text-white border border-slate-700 rounded-xl text-xs font-bold transition-all active:scale-95",
                                                onclick: move |_| {
                                                    if let Ok(conn) = init_db() {
                                                        let _ = update_task_status(&conn, instance_id_undo, "ongoing");
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
                    })
                }
            }
        }
    }
}
