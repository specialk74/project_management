use chrono::{Datelike, Local};
use slint::SharedString;

slint::include_modules!();

fn dates(start_date: &mut chrono::DateTime<Local>) -> Vec<SharedString> {
    let mut dates: Vec<SharedString> = Vec::new();
    *start_date = primo_giorno_settimana_corrente(start_date);
    let week_number = 52 - start_date.iso_week().week();
    for _ in 0..52 + week_number {
        let date_str = start_date.format("%y-%m-%d").to_string();
        dates.push(date_str.clone().into());
        *start_date += chrono::Duration::days(7);
    }
    dates
}

fn primo_giorno_settimana_corrente(data: &chrono::DateTime<Local>) -> chrono::DateTime<Local> {
    let giorni_da_lunedi = data.weekday().num_days_from_monday();
    *data - chrono::Duration::days(giorni_da_lunedi as i64)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
fn main() {
    use slint::Model;

    let main_window = MainWindow::new().unwrap();

    let mut start_date = Local::now() - chrono::Duration::days(30);
    let model = std::rc::Rc::new(slint::VecModel::from(dates(&mut start_date)));

    let worker_efforts: Vec<SharedString> = vec![
        "Worker12345|10",
        "Worker12|20",
        "Worker1|10\nWorker9|10",
        "Worker54321|10",
        "",
        "Worker5|25",
        "Worker|15",
        "Worker3|15",
        "Worker4|65",
        "Worker6|88",
        "Worker789|92",
        "Worker8|78",
        "Worker9|82",
    ]
    .iter()
    .map(|s| SharedString::from(*s))
    .collect();
    let worker_efforts_model = std::rc::Rc::new(slint::VecModel::from(worker_efforts));

    let mut partial_efforts = vec![];
    let mut effort = 0;
    for worker_effort in worker_efforts_model.iter() {
        let parts: Vec<&str> = worker_effort.split('\n').collect();
        if parts.is_empty() {
            partial_efforts.push(effort);
            continue;
        }
        for worker in parts.iter() {
            let sub_parts: Vec<&str> = worker.split('|').collect();
            if sub_parts.len() == 2 {
                if let Ok(partial_effort) = sub_parts[1].parse::<i32>() {
                    effort += partial_effort;
                }
            }
        }
        partial_efforts.push(effort);
    }

    let partial_efforts_model = std::rc::Rc::new(slint::VecModel::from(partial_efforts));

    let pjm_datas = PjmDatas {
        current_dates: model.clone().into(),
        current_week: SharedString::from(
            primo_giorno_settimana_corrente(&Local::now())
                .format("%y-%m-%d")
                .to_string(),
        ),
        dev_efforts: std::rc::Rc::new(slint::VecModel::from(vec![DevEffort {
            worker_efforts: worker_efforts_model.clone().into(),
            partial_efforts: partial_efforts_model.clone().into(),
            effort: 250,
            background_color: slint::Color::from_rgb_u8(0xcc, 0xdd, 0xcc),
        }]))
        .into(),
    };
    main_window.set_pjm_datas(pjm_datas);

    main_window.run().unwrap();
}
