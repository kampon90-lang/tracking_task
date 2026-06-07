use crate::database::{
    delete_template, get_recurring_templates, init_db, update_template, TaskTemplate,
};
use dioxus::prelude::*;

#[component]
pub fn RecurringList() -> Element {
    let mut templates = use_signal(|| Vec::<TaskTemplate>::new());
    let mut editing_id = use_signal(|| None::<i32>);
    let mut edit_title = use_signal(String::new);
    let mut edit_note = use_signal(String::new);

    let mut load_templates = move || {
        if let Ok(conn) = init_db() {
            if let Ok(data) = get_recurring_templates(&conn) {
                templates.set(data);
            }
        }
    };

    use_effect(move || {
        load_templates();
    });

    rsx! {
        div { class: "flex flex-col gap-4 max-h-[70vh] overflow-y-auto pr-2 custom-scrollbar",
            if templates.read().is_empty() {
                div { class: "text-center py-10 text-slate-500 font-bold", "📭 NO RECURRING TASKS" }
            } else {
                {
                    templates.read().iter().map(|template| {
                        let is_editing = editing_id() == Some(template.id);
                        let template_id = template.id;
                        let t_title = template.title.clone();
                        let t_note = template.note.clone();

                        rsx! {
                            div { key: "{template.id}", class: "p-5 rounded-2xl border border-slate-700 bg-slate-800 transition-all",
                                if is_editing {
                                    div { class: "flex flex-col gap-3",
                                        input { class: "w-full px-2 py-1 bg-slate-950 border border-slate-700 rounded text-white", value: "{edit_title}", oninput: move |e| edit_title.set(e.value()) }
                                        textarea { class: "w-full px-2 py-1 bg-slate-950 border border-slate-700 rounded text-slate-300 text-xs h-16 resize-none", value: "{edit_note}", oninput: move |e| edit_note.set(e.value()) }
                                        div { class: "flex justify-end gap-2",
                                            button { class: "px-3 py-1 bg-slate-700 text-white rounded-lg text-xs", onclick: move |_| editing_id.set(None), "Cancel" }
                                            button { class: "px-3 py-1 bg-blue-600 text-white rounded-lg text-xs", onclick: move |_| {
                                                if let Ok(conn) = init_db() {
                                                    // 💡 แก้ไขต้นแบบ (กระทบทุกงาน)
                                                    let _ = update_template(&conn, template_id, &edit_title(), &edit_note());
                                                    load_templates();
                                                    editing_id.set(None);
                                                }
                                            }, "💾 Save" }
                                        }
                                    }
                                } else {
                                    div { class: "flex flex-col gap-2",
                                        div { class: "flex justify-between items-start",
                                            h3 { class: "font-black text-white text-lg", "{template.title}" }
                                            span { class: "px-2 py-1 bg-slate-900 text-slate-300 rounded-lg text-[10px] font-bold border border-slate-700 whitespace-nowrap", "🔄 {template.recurrence}" }
                                        }
                                        if !template.note.is_empty() {
                                            p { class: "text-xs text-slate-400 mt-1", "{template.note}" }
                                        }
                                        div { class: "flex justify-end gap-2 mt-3 pt-3 border-t border-slate-700/50",
                                            button { class: "px-3 py-1.5 bg-orange-900/20 text-orange-400 border border-orange-800/50 rounded-xl text-xs font-bold", onclick: move |_| {
                                                edit_title.set(t_title.clone());
                                                edit_note.set(t_note.clone());
                                                editing_id.set(Some(template_id));
                                            }, "✏️ Edit" }
                                            button { class: "px-3 py-1.5 bg-red-900/20 text-red-500 border border-red-800/50 rounded-xl text-xs font-bold", onclick: move |_| {
                                                if let Ok(conn) = init_db() {
                                                    // 💡 ลบต้นแบบ (กระทบทุกงาน)
                                                    let _ = delete_template(&conn, template_id);
                                                    load_templates();
                                                }
                                            }, "🗑️ Delete All" }
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
