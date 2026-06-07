use crate::database::{get_tasks_by_date, init_db, update_task_status, TaskInstance, TaskTemplate};
use chrono::Local;
use dioxus::prelude::*;

#[component]
pub fn Calendar() -> Element {
    // 1. ดึงวันที่ปัจจุบันมาเป็นค่าเริ่มต้น
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut selected_date = use_signal(|| today);

    // State สำหรับเก็บรายการงานของวันนั้น
    let mut tasks = use_signal(|| Vec::<(TaskInstance, TaskTemplate)>::new());

    // ฟังก์ชันสำหรับโหลดงานตามวันที่กำหนด
    let mut load_tasks_for_date = move |date: String| {
        if let Ok(conn) = init_db() {
            if let Ok(data) = get_tasks_by_date(&conn, &date) {
                tasks.set(data);
            } else {
                tasks.set(Vec::new());
            }
        }
    };

    // 2. สั่งโหลดข้อมูลใหม่ทุกครั้งที่ selected_date เปลี่ยนแปลง
    use_effect(move || {
        load_tasks_for_date(selected_date());
    });

    rsx! {
        div { class: "flex flex-col gap-6",

            // --- แผงเลือกวันที่ (Date Selector) ---
            div { class: "bg-slate-900 p-5 rounded-2xl border border-slate-800 shadow-lg flex items-center gap-5",
                div { class: "text-4xl", "📅" }
                div { class: "flex-1",
                    label { class: "text-[10px] font-bold uppercase tracking-widest text-slate-500 mb-1 block", "Selected Date" }
                    input {
                        // บังคับให้ปฏิทินเป็นสีขาวด้วย inline CSS
                        class: "w-full bg-transparent text-white font-black text-2xl tracking-wide focus:outline-none cursor-pointer [color-scheme:dark]",
                        r#type: "date",
                        value: "{selected_date}",
                        onchange: move |e| selected_date.set(e.value())
                    }
                }
            }

            // --- รายการ Checklist ของวันนั้น ---
            div { class: "flex flex-col gap-3 max-h-[55vh] overflow-y-auto pr-2 custom-scrollbar",
                if tasks.read().is_empty() {
                    div { class: "text-center py-12 text-slate-600 font-bold tracking-widest bg-slate-900/30 rounded-2xl border-2 border-dashed border-slate-800/50 mt-2",
                        "✨ FREE DAY! NO TASKS ✨"
                    }
                } else {
                    {
                        tasks.read().iter().map(|(instance, template)| {
                            let is_finished = instance.status == "finished";

                            // เปลี่ยนสีสไตล์ตามสถานะ
                            let title_class = if is_finished { "text-slate-500 line-through decoration-slate-600" } else { "text-slate-100" };
                            let bg_color = if is_finished { "bg-slate-900/30" } else { "bg-slate-900" };
                            let border_color = if is_finished { "border-emerald-900/20" } else { "border-slate-800" };

                            // จัดการคลาสของปุ่มวงกลม
                            let check_btn_class = if is_finished {
                                "bg-emerald-500 border-emerald-500"
                            } else {
                                "border-slate-600 hover:border-blue-400 bg-slate-800"
                            };

                            let instance_id = instance.id;

                            rsx! {
                                div { key: "{instance.id}", class: "p-4 rounded-xl border flex items-center gap-4 transition-all shadow-sm {bg_color} {border_color}",

                                    // 1. ปุ่ม Checkbox ทรงวงกลม
                                    button {
                                        class: "w-7 h-7 rounded-full border-2 flex items-center justify-center transition-all flex-shrink-0 {check_btn_class}",
                                        onclick: move |_| {
                                            if let Ok(conn) = init_db() {
                                                // สลับสถานะ Finished <-> Ongoing
                                                let new_status = if is_finished { "ongoing" } else { "finished" };
                                                let _ = update_task_status(&conn, instance_id, new_status);
                                                load_tasks_for_date(selected_date());
                                            }
                                        },
                                        if is_finished {
                                            span { class: "text-white text-[12px] font-black", "✓" }
                                        }
                                    }

                                    // 2. ชื่อและโน้ตของงาน
                                    div { class: "flex-1 min-w-0 flex flex-col justify-center",
                                        h4 { class: "font-bold text-sm truncate {title_class}", "{template.title}" }
                                        if !template.note.is_empty() {
                                            p { class: "text-[11px] text-slate-500 truncate mt-0.5", "{template.note}" }
                                        }
                                    }

                                    // 3. ป้ายบอกว่าเป็นงานทำซ้ำ
                                    if template.recurrence != "none" {
                                        div { class: "px-2 py-1 bg-slate-800/80 rounded-lg border border-slate-700/50 flex items-center justify-center",
                                            span { class: "text-[10px]", "🔄" }
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
}
