use chrono::{Datelike, Duration, Months, NaiveDate, Weekday};
use dioxus::html::template;
use rusqlite::{Connection, Result};

#[derive(Clone, Debug)]
pub struct TaskTemplate {
    pub id: i32,
    pub title: String,
    pub note: String,
    pub recurrence: String,
}

#[derive(Clone, Debug)]
pub struct TaskInstance {
    pub id: i32,
    pub template_id: i32,
    pub due_date: String,
    pub status: String,
}

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("tasks.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            note TEXT,
            recurrence TEXT DEFAULT 'none'
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_instances (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id INTEGER,
            due_date TEXT NOT NULL,
            status TEXT DEFAULT 'ongoing',
            FOREIGN KEY(template_id) REFERENCES task_templates(id)
        )",
        (),
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_due_date ON task_instances (due_date)",
        (),
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_template_id ON task_instances (template_id)",
        (),
    )?;

    Ok(conn)
}

pub fn run_maintenance(conn: &Connection) -> Result<()> {
    conn.execute("VACUUM", ())?;
    conn.execute("ANALYZE", ())?;
    Ok(())
}

pub fn add_new_task(
    conn: &Connection,
    title: &str,
    note: &str,
    recurrence_rule: &str,
    start_date: &str,
) -> Result<i32> {
    conn.execute(
        "INSERT INTO task_templates (title, note, recurrence) VALUES (?1, ?2, ?3)",
        (title, note, recurrence_rule),
    )?;

    let template_id = conn.last_insert_rowid() as i32;
    let generated_dates = generate_recurring_dates(start_date, recurrence_rule);

    for date in generated_dates {
        conn.execute(
            "
            INSERT INTO task_instances (template_id, due_date, status) VALUES (?1, ?2, 'ongoing')
            ",
            (template_id, date),
        )?;
    }

    Ok(template_id)
}

fn generate_recurring_dates(start_date_str: &str, rule: &str) -> Vec<String> {
    let start_date = NaiveDate::parse_from_str(start_date_str, "%Y-%m-%d").unwrap_or_default();
    let mut dates = Vec::new();

    if rule == "none" || rule.is_empty() {
        dates.push(start_date_str.to_string());
        return dates;
    }

    let limit = start_date + Duration::days(365);
    let parts: Vec<&str> = rule.split(':').collect();

    match parts[0] {
        "days" => {
            if let Ok(interval) = parts[1].parse::<i64>() {
                let mut current = start_date;
                while current <= limit {
                    dates.push(current.format("%Y-%m-%d").to_string());
                    current += Duration::days(interval);
                }
            }
        }

        "months" => {
            if let Ok(interval) = parts[1].parse::<u32>() {
                let mut current = start_date;
                while current <= limit {
                    dates.push(current.format("%Y-%m-%d").to_string());
                    if let Some(next_month) = current.checked_add_months(Months::new(interval)) {
                        current = next_month;
                    } else {
                        break;
                    }
                }
            }
        }

        "nth" => {
            if parts.iter().len() == 3 {
                let n: u32 = parts[1].parse().unwrap_or(1);
                let target_wd = match parts[2] {
                    "Mon" => Weekday::Mon,
                    "Tue" => Weekday::Tue,
                    "Wed" => Weekday::Wed,
                    "Thu" => Weekday::Thu,
                    "Fri" => Weekday::Fri,
                    "Sat" => Weekday::Sat,
                    "Sun" => Weekday::Mon,
                    _ => Weekday::Mon,
                };

                let mut current_month = start_date;
                for _ in 0..12 {
                    if let Some(first_day) =
                        NaiveDate::from_ymd_opt(current_month.year(), current_month.month(), 1)
                    {
                        let mut first_target_date = first_day;

                        while first_target_date.weekday() != target_wd {
                            first_target_date += Duration::days(1);
                        }

                        let final_date = first_target_date + Duration::days(((n - 1) * 7) as i64);
                        if final_date.month() == current_month.month() && final_date >= start_date {
                            dates.push(final_date.format("%Y-%m-%d").to_string());
                        }
                    }

                    if let Some(next_month) = current_month.checked_add_months(Months::new(1)) {
                        current_month = next_month;
                    } else {
                        break;
                    }
                }
            }
        }
        _ => {
            dates.push(start_date_str.to_string());
        }
    }
    dates
}

pub fn get_tasks_by_date(
    conn: &Connection,
    date: &str,
) -> Result<Vec<(TaskInstance, TaskTemplate)>> {
    let mut stmt = conn.prepare(
        "SELECT i.id, i.template_id, i.due_date, i.status,
                t.id, t.title, t.note, t.recurrence
         FROM task_instances i
         JOIN task_templates t ON i.template_id = t.id
         WHERE i.due_date = ?1",
    )?;

    let rows = stmt.query_map([date], |row| {
        Ok((
            TaskInstance {
                id: row.get(0)?,
                template_id: row.get(1)?,
                due_date: row.get(2)?,
                status: row.get(3)?,
            },
            TaskTemplate {
                id: row.get(4)?,
                title: row.get(5)?,
                note: row.get(6)?,
                recurrence: row.get(7)?,
            },
        ))
    })?;

    Ok(rows.filter_map(Result::ok).collect())
}

pub fn update_task_status(conn: &Connection, instance_id: i32, status: &str) -> Result<()> {
    conn.execute(
        "UPDATE task_instances SET status = ?1 WHERE id = ?2",
        (status, instance_id),
    )?;
    Ok(())
}

pub fn get_all_tasks(conn: &Connection) -> Result<Vec<(TaskInstance, TaskTemplate)>> {
    let mut stmt = conn.prepare(
        "SELECT i.id, i.template_id, i.due_date, i.status,
                t.id, t.title, t.note, t.recurrence
         FROM task_instances i
         JOIN task_templates t ON i.template_id = t.id
         ORDER BY i.due_date ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            TaskInstance {
                id: row.get(0)?,
                template_id: row.get(1)?,
                due_date: row.get(2)?,
                status: row.get(3)?,
            },
            TaskTemplate {
                id: row.get(4)?,
                title: row.get(5)?,
                note: row.get(6)?,
                recurrence: row.get(7)?,
            },
        ))
    })?;

    Ok(rows.filter_map(Result::ok).collect())
}

pub fn postpone_task(conn: &Connection, instance_id: i32, new_date: &str) -> Result<()> {
    conn.execute(
        "
            UPDATE task_instances SET due_date = ?1 WHERE id = ?2
        ",
        (new_date, instance_id),
    )?;
    Ok(())
}

pub fn update_task_details(
    conn: &Connection,
    instance_id: i32,
    template_id: i32,
    new_title: &str,
    new_note: &str,
    new_due_date: &str,
) -> Result<()> {
    conn.execute(
        "
        UPDATE task_templates SET title = ?1, note = ?2 WHERE id =?3
        ",
        (new_title, new_note, template_id),
    )?;

    conn.execute(
        "
        UPDATE task_instances SET due_date = ?1 WHERE id = ?2
        ",
        (new_due_date, instance_id),
    )?;
    Ok(())
}

pub fn delete_task(conn: &Connection, template_id: i32) -> Result<()> {
    conn.execute(
        "
            DELETE FROM task_instances WHERE template_id = ?1
        ",
        [template_id],
    )?;

    conn.execute(
        "
            DELETE FROM task_templates WHERE id = ?1
        ",
        [template_id],
    )?;
    Ok(())
}

pub fn delete_single_instance(conn: &Connection, instance_id: i32) -> Result<()> {
    conn.execute("DELETE FROM task_instances WHERE id = ?1", [instance_id])?;
    Ok(())
}

pub fn update_single_instance(
    conn: &Connection,
    instance_id: i32,
    new_title: &str,
    new_note: &str,
    new_due_date: &str,
) -> Result<()> {
    // 💡 สร้าง Template ใหม่แบบ 'Once' (none) เพื่อดึงงานนี้ให้หลุดออกจากกลุ่ม Recurring เดิม
    conn.execute(
        "INSERT INTO task_templates (title, note, recurrence) VALUES (?1, ?2, 'none')",
        (new_title, new_note),
    )?;
    let new_template_id = conn.last_insert_rowid() as i32;

    // ผูกงานนี้กับ Template ใหม่ พร้อมอัปเดตวันที่
    conn.execute(
        "UPDATE task_instances SET template_id = ?1, due_date = ?2 WHERE id = ?3",
        (new_template_id, new_due_date, instance_id),
    )?;
    Ok(())
}

// ==========================================
// 2. ฟังก์ชันจัดการ "ต้นแบบงานซ้ำ" (ลบ/แก้ ทั้งหมด)
// ==========================================

pub fn get_recurring_templates(conn: &Connection) -> Result<Vec<TaskTemplate>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, note, recurrence FROM task_templates WHERE recurrence != 'none' ORDER BY id DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(TaskTemplate {
            id: row.get(0)?,
            title: row.get(1)?,
            note: row.get(2)?,
            recurrence: row.get(3)?,
        })
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn delete_template(conn: &Connection, template_id: i32) -> Result<()> {
    // ลบงานย่อยในอนาคตทั้งหมดที่เกิดจากแม่พิมพ์นี้
    conn.execute(
        "DELETE FROM task_instances WHERE template_id = ?1",
        [template_id],
    )?;
    // ลบแม่พิมพ์ทิ้ง
    conn.execute("DELETE FROM task_templates WHERE id = ?1", [template_id])?;
    Ok(())
}

pub fn update_template(
    conn: &Connection,
    template_id: i32,
    new_title: &str,
    new_note: &str,
) -> Result<()> {
    // เปลี่ยนชื่อ/โน้ตของแม่พิมพ์ ซึ่งจะไปกระทบงานย่อยที่โชว์อยู่ทั้งหมดทันที
    conn.execute(
        "UPDATE task_templates SET title = ?1, note = ?2 WHERE id = ?3",
        (new_title, new_note, template_id),
    )?;
    Ok(())
}
