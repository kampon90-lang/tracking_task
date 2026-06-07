use crate::database::{
    delete_single_instance, get_all_tasks, init_db, postpone_task, update_single_instance,
    update_task_status, TaskInstance, TaskTemplate,
};
use chrono::Local;
use dioxus::prelude::*;

#[component]
pub fn TaskList() -> Element {
    let mut tasks = use_signal(|| Vec::<(TaskInstance, TaskTemplate)>::new());
    let mut search_query = use_signal(String::new);
    let mut sort_by = use_signal(|| "date".to_string());

    let mut editing_id = use_signal(|| None::<i32>);
    let mut edit_title = use_signal(String::new);
    let mut edit_note = use_signal(String::new);
    let mut edit_date = use_signal(String::new);

    let mut postponing_id = use_signal(|| None::<i32>);
    let mut postpone_date = use_signal(String::new); // ✅ dedicated signal

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

    let mut filtered_tasks = tasks.read().clone();
    filtered_tasks.retain(|(_, template)| {
        template
            .title
            .to_lowercase()
            .contains(&search_query().to_lowercase())
    });
    filtered_tasks.sort_by(|(a, _), (b, _)| {
        if sort_by() == "date" {
            a.due_date.cmp(&b.due_date)
        } else {
            a.id.cmp(&b.id)
        }
    });

    rsx! {
        div { class: "flex flex-col gap-4 max-h-[70vh] overflow-y-auto pr-2 custom-scrollbar",
            div { class: "flex gap-2 mb-2",
                input {
                    class: "flex-1 px-4 py-3 bg-slate-900 border border-slate-700 rounded-xl text-white focus:ring-2 focus:ring-blue-500",
                    placeholder: "🔍 ค้นหางาน...",
                    oninput: move |e| search_query.set(e.value())
                }
                select {
                    class: "px-3 bg-slate-900 border border-slate-700 rounded-xl text-white",
                    onchange: move |e| sort_by.set(e.value()),
                    option { value: "date", "วันที่" }
                    option { value: "id", "ล่าสุด" }
                }
            }

            {
                filtered_tasks.into_iter().map(|(instance, template)| {
                    let is_editing = editing_id() == Some(instance.id);
                    let is_postponing = postponing_id() == Some(instance.id);
                    let is_finished = instance.status == "finished";
                    let is_overdue = !is_finished && instance.due_date < Local::now().format("%Y-%m-%d").to_string();

                    let bg_color = if is_finished { "bg-slate-900/50" } else if is_overdue { "bg-red-950/20" } else { "bg-slate-900" };
                    let border_color = if is_finished { "border-emerald-900" } else if is_overdue { "border-red-900" } else { "border-slate-800" };
                    let title_class = if is_finished { "line-through text-slate-500" } else { "text-white" };
                    let date_class = if is_overdue { "text-red-500" } else { "text-blue-400" };

                    let instance_id = instance.id;
                    let template_title = template.title.clone();
                    let template_note = template.note.clone();
                    let instance_date = instance.due_date.clone();

                    rsx! {
                        div { key: "{instance.id}", class: "p-5 rounded-2xl border {border_color} {bg_color} transition-all",
                            if is_editing {
                                div { class: "flex flex-col gap-3",
                                    input { class: "w-full px-2 py-1 bg-slate-950 border border-slate-700 rounded text-white", value: "{edit_title}", oninput: move |e| edit_title.set(e.value()) }
                                    textarea { class: "w-full px-2 py-1 bg-slate-950 border border-slate-700 rounded text-slate-300 text-xs", value: "{edit_note}", oninput: move |e| edit_note.set(e.value()) }
                                    div { class: "flex justify-between",
                                        input { class: "bg-slate-950 text-white p-1 rounded [color-scheme:dark]", r#type: "date",
                                            value: "{instance_date}",
                                            onchange: move |e| edit_date.set(e.value())
                                        }
                                        div { class: "flex gap-2",
                                            button { onclick: move |_| editing_id.set(None), "Cancel" }
                                            button { class: "text-blue-400 font-bold", onclick: move |_| {
                                                if let Ok(conn) = init_db() {
                                                    let _ = update_single_instance(&conn, instance_id, &edit_title(), &edit_note(), &edit_date());
                                                    load_tasks();
                                                    editing_id.set(None);
                                                }
                                            }, "Save" }
                                        }
                                    }
                                }
                            } else {
                                div { class: "flex flex-col gap-2",
                                    h3 { class: "font-black {title_class}", "{template.title}" }
                                    if !template.note.is_empty() {
                                        p { class: "text-xs text-slate-400", "{template.note}" }
                                    }
                                    div { class: "flex justify-between items-center mt-2",
                                        span { class: "text-xs font-bold {date_class}", "📅 {instance.due_date}" }
                                        div { class: "flex gap-2 items-center",
                                            if !is_finished {
                                                if is_postponing {
                                                    div { class: "flex items-center gap-1",
                                                        input {
                                                            class: "bg-slate-950 text-orange-400 p-1 rounded text-xs font-bold [color-scheme:dark]",
                                                            r#type: "date",
                                                            autofocus: true,
                                                            value: "{postpone_date}", // ✅ bound value
                                                            oninput: move |e| postpone_date.set(e.value()) // ✅ oninput
                                                        }
                                                        button { class: "text-[10px] bg-emerald-900/30 text-emerald-400 border border-emerald-800 rounded px-2 py-1",
                                                            onclick: move |_| {
                                                                if !postpone_date().is_empty() {
                                                                    if let Ok(conn) = init_db() {
                                                                        let _ = postpone_task(&conn, instance_id, &postpone_date()); // ✅ own signal
                                                                        load_tasks();
                                                                        postponing_id.set(None);
                                                                        postpone_date.set(String::new()); // ✅ cleanup
                                                                    }
                                                                } else {
                                                                    postponing_id.set(None);
                                                                }
                                                            },
                                                            "✔️"
                                                        }
                                                        button { class: "text-[10px] text-slate-500 hover:text-white px-1",
                                                            onclick: move |_| {
                                                                postponing_id.set(None);
                                                                postpone_date.set(String::new()); // ✅ cleanup on cancel
                                                            },
                                                            "❌"
                                                        }
                                                    }
                                                } else {
                                                    button { onclick: move |_| {
                                                        postpone_date.set(String::new()); // ✅ clear own signal
                                                        postponing_id.set(Some(instance_id));
                                                    }, "⏳" }
                                                }

                                                if !is_postponing {
                                                    button { onclick: move |_| {
                                                        edit_title.set(template_title.clone());
                                                        edit_note.set(template_note.clone());
                                                        edit_date.set(instance_date.clone());
                                                        editing_id.set(Some(instance_id));
                                                    }, "✏️" }
                                                    button { onclick: move |_| {
                                                        if let Ok(conn) = init_db() {
                                                            let _ = update_task_status(&conn, instance_id, "finished");
                                                            load_tasks();
                                                        }
                                                    }, "✔️" }
                                                }
                                            }
                                            button { class: "text-red-500", onclick: move |_| {
                                                if let Ok(conn) = init_db() {
                                                    let _ = delete_single_instance(&conn, instance_id);
                                                    load_tasks();
                                                }
                                            }, "🗑️" }
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
