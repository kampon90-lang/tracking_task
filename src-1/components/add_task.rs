use crate::database::{add_new_task, init_db};
use dioxus::prelude::*;

#[component]
pub fn AddTask() -> Element {
    let mut title = use_signal(String::new);
    let mut note = use_signal(String::new);
    let mut due_date = use_signal(|| String::new());

    // State สำหรับระบบความถี่อัจฉริยะ
    let mut rec_mode = use_signal(|| "none".to_string());
    let mut custom_num = use_signal(|| "1".to_string());
    let mut nth_week = use_signal(|| "1".to_string());
    let mut nth_day = use_signal(|| "Mon".to_string());

    // 💡 ดึง Logic การเช็คคำว่า Days/Months ออกมาเป็นตัวแปร เพื่อป้องกัน Error parse string
    let duration_label = if rec_mode() == "days" {
        "Days"
    } else {
        "Months"
    };

    rsx! {
        div { class: "flex flex-col gap-6 p-2",

            // --- Work Name & Note ---
            div { class: "flex flex-col gap-2",
                label { class: "text-xs font-bold uppercase tracking-widest text-slate-500 ml-1", "Work Name" }
                input {
                    class: "w-full px-4 py-3 bg-white border border-slate-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500 text-slate-900 transition-all shadow-inner",
                    placeholder: "ระบุชื่องาน...",
                    value: "{title}",
                    oninput: move |e| title.set(e.value())
                }
            }

            div { class: "flex flex-col gap-2",
                label { class: "text-xs font-bold uppercase tracking-widest text-slate-500 ml-1", "Note / Description" }
                textarea {
                    class: "w-full px-4 py-3 bg-white border border-slate-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500 text-slate-900 h-24 resize-none transition-all shadow-inner",
                    placeholder: "รายละเอียด...",
                    value: "{note}",
                    oninput: move |e| note.set(e.value())
                }
            }

            // --- Date & Recurrence Type ---
            div { class: "grid grid-cols-2 gap-4",
                div { class: "flex flex-col gap-2",
                    label { class: "text-xs font-bold uppercase tracking-widest text-slate-500 ml-1", "Start Date" }
                    input {
                        class: "field-modern cursor-pointer [color-scheme:dark]",
                        r#type: "date", value: "{due_date}", oninput: move |e| due_date.set(e.value())
                    }
                }
                div { class: "flex flex-col gap-2",
                    label { class: "text-xs font-bold uppercase tracking-widest text-slate-500 ml-1", "Recurrence Mode" }
                    select {
                        class: "field-modern cursor-pointer",
                        onchange: move |e| rec_mode.set(e.value()),
                        option { value: "none", "Once (ทำครั้งเดียว)" }
                        option { value: "days", "Every N Days (ทุกๆ N วัน)" }
                        option { value: "months", "Every N Months (ทุกๆ N เดือน)" }
                        option { value: "nth", "Specific Weekday (ระบุวันในสัปดาห์)" }
                    }
                }
            }

            // --- 💡 Dynamic UI: แผงควบคุมจะเปลี่ยนไปตาม Mode ที่เลือก ---
            if rec_mode() == "days" || rec_mode() == "months" {
                div { class: "flex items-center gap-4 bg-slate-900/50 p-4 rounded-xl border border-slate-800",
                    label { class: "text-sm font-bold text-slate-300", "Repeat every:" }
                    input {
                        class: "w-20 px-3 py-2 bg-slate-950 border border-slate-700 rounded-lg text-white text-center focus:ring-2 focus:ring-blue-500",
                        r#type: "number", min: "1", value: "{custom_num}", oninput: move |e| custom_num.set(e.value())
                    }
                    // 💡 เรียกใช้ตัวแปรแทนการเขียน if-else ลงไปตรงๆ
                    span { class: "text-sm font-bold text-slate-400", "{duration_label}" }
                }
            } else if rec_mode() == "nth" {
                div { class: "flex items-center gap-3 bg-slate-900/50 p-4 rounded-xl border border-slate-800",
                    label { class: "text-sm font-bold text-slate-300", "Every:" }
                    select { class: "flex-1 px-3 py-2 bg-slate-950 border border-slate-700 rounded-lg text-white focus:ring-2 focus:ring-blue-500",
                        onchange: move |e| nth_week.set(e.value()),
                        option { value: "1", "1st (แรก)" }
                        option { value: "2", "2nd (ที่สอง)" }
                        option { value: "3", "3rd (ที่สาม)" }
                        option { value: "4", "4th (ที่สี่)" }
                        option { value: "5", "5th (สุดท้าย)" }
                    }
                    select { class: "flex-1 px-3 py-2 bg-slate-950 border border-slate-700 rounded-lg text-white focus:ring-2 focus:ring-blue-500",
                        onchange: move |e| nth_day.set(e.value()),
                        option { value: "Mon", "Monday" }
                        option { value: "Tue", "Tuesday" }
                        option { value: "Wed", "Wednesday" }
                        option { value: "Thu", "Thursday" }
                        option { value: "Fri", "Friday" }
                        option { value: "Sat", "Saturday" }
                        option { value: "Sun", "Sunday" }
                    }
                }
            }

            // --- Save Button ---
            div { class: "pt-4",
                button {
                    class: "btn-save-modern",
                    onclick: move |_| {
                        if !title().is_empty() && !due_date().is_empty() {
                            let rule = match rec_mode().as_str() {
                                "days" => format!("days:{}", custom_num()),
                                "months" => format!("months:{}", custom_num()),
                                "nth" => format!("nth:{}:{}", nth_week(), nth_day()),
                                _ => "none".to_string()
                            };

                            if let Ok(conn) = init_db() {
                                let _ = add_new_task(&conn, &title(), &note(), &rule, &due_date());

                                title.set("".to_string());
                                note.set("".to_string());
                                due_date.set("".to_string());
                                rec_mode.set("none".to_string());
                            }
                        }
                    },
                    span { "💾 SAVE TASK" }
                }
            }
        } // 💡 แก้ไข </div> ให้เป็น } ปิด Block ของ div หลัก
    }
}
