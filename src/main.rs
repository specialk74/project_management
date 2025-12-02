use chrono::{Datelike, Local};
use slint::{Model, SharedString, VecModel};

slint::include_modules!();

fn dates(start_date: &chrono::DateTime<Local>) -> Vec<SharedString> {
    let mut dates: Vec<SharedString> = Vec::new();
    let mut start_date = primo_giorno_settimana_corrente(start_date);

    let week_number = 52 - start_date.iso_week().week();
    for _ in 0..52 + week_number {
        let date_str = start_date.format("%y-%m-%d").to_string();
        dates.push(date_str.clone().into());
        start_date += chrono::Duration::days(7);
        start_date = primo_giorno_settimana_corrente(&start_date);
    }
    dates
}

fn primo_giorno_settimana_corrente(data: &chrono::DateTime<Local>) -> chrono::DateTime<Local> {
    let giorni_da_lunedi = data.weekday().num_days_from_monday();
    *data - chrono::Duration::days(giorni_da_lunedi as i64)
}

fn get_partial_efforts_model(worker_efforts: &Vec<&str>) -> std::rc::Rc<VecModel<i32>> {
    let mut partial_efforts = vec![];
    let mut effort = 0;
    for worker_effort in worker_efforts.iter() {
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

    std::rc::Rc::new(slint::VecModel::from(partial_efforts))
}

fn get_max_height(worker_efforts: &Vec<&str>) -> i32 {
    let mut effort = 0;
    for worker_effort in worker_efforts.iter() {
        let count = worker_effort.split('\n').count() as i32;
        if effort < count {
            effort = count;
        }
    }
    effort
}

fn get_worker_efforts_model(worker_efforts: &Vec<&str>) -> std::rc::Rc<VecModel<SharedString>> {
    let shared_string: Vec<SharedString> = worker_efforts
        .iter()
        .map(|s| SharedString::from(*s))
        .collect();
    std::rc::Rc::new(slint::VecModel::from(shared_string))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
// Esempio di come mantenere riferimenti ai modelli interni
fn setup_shared_models() -> (
    std::rc::Rc<VecModel<SharedString>>,
    std::rc::Rc<VecModel<i32>>,
) {
    let worker_efforts = vec!["SharedWorker|50"];
    let worker_model = get_worker_efforts_model(&worker_efforts);
    let partial_model = get_partial_efforts_model(&worker_efforts);
    (worker_model, partial_model)
}

fn main() {
    let main_window = MainWindow::new().unwrap();
    let app_state = main_window.global::<AppState>();

    let start_date = Local::now() - chrono::Duration::days(30);
    let model = std::rc::Rc::new(slint::VecModel::from(dates(&start_date)));

    let worker_efforts_1 = vec![
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
    ];

    let worker_efforts_2 = vec![
        "Worker12345|80",
        "Worker12|80",
        "Worker54321|80",
        "Worker1|80\nWorker9|80\nWorker1|90",
        "Worker5|90",
        "Worker5|60",
        "Worker|25",
        "Worker3|45",
        "Worker4|65",
        "Worker6|88",
        "Worker789|92",
        "Worker8|78",
        "Worker9|82",
    ];

    // Mantieni un riferimento ai modelli per poterli modificare
    let dev_efforts_model = std::rc::Rc::new(slint::VecModel::from(vec![
        DevEffort {
            worker_efforts: get_worker_efforts_model(&worker_efforts_1).clone().into(),
            partial_efforts: get_partial_efforts_model(&worker_efforts_1).clone().into(),
            max_height: get_max_height(&worker_efforts_1) + 1,
            effort: 250,
            background_color: slint::Color::from_rgb_u8(0xcc, 0xdd, 0xcc),
        },
        DevEffort {
            worker_efforts: get_worker_efforts_model(&worker_efforts_2).clone().into(),
            partial_efforts: get_partial_efforts_model(&worker_efforts_2).clone().into(),
            max_height: get_max_height(&worker_efforts_2) + 1,
            effort: 1000,
            background_color: slint::Color::from_rgb_u8(0xdd, 0xcc, 0xdd),
        },
    ]));

    // Esempio: mantieni anche riferimenti ai modelli interni per modifiche più granulari
    let (shared_worker_model, shared_partial_model) = setup_shared_models();

    let pjm_datas = PjmDatas {
        project_name: "Project Name Lungo".into(),
        current_dates: model.clone().into(),
        current_week: SharedString::from(
            primo_giorno_settimana_corrente(&Local::now())
                .format("%y-%m-%d")
                .to_string(),
        ),
        dev_efforts: dev_efforts_model.clone().into(),
    };

    main_window.set_pjm_datas(pjm_datas);

    // METODO 1: Modifica diretta tramite modelli condivisi (MIGLIORE)
    let dev_efforts_ref = dev_efforts_model.clone();
    let dates_ref = model.clone();
    let main_window_weak = main_window.as_weak();
    let worker_ref = shared_worker_model.clone();
    let partial_ref = shared_partial_model.clone();

    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(2),
        move || {
            // OPZIONE A: Modifica diretta del VecModel - automaticamente aggiorna l'UI!
            /*
            if dev_efforts_ref.row_count() < 5 {
                let new_effort = DevEffort {
                    worker_efforts: get_worker_efforts_model(&vec!["AutoWorker|123"]).into(),
                    partial_efforts: get_partial_efforts_model(&vec!["AutoWorker|123"]).into(),
                    effort: 123,
                    background_color: slint::Color::from_rgb_u8(0xff, 0xcc, 0xcc),
                };
                dev_efforts_ref.push(new_effort); // ← Aggiornamento automatico!
            }
            */

            // OPZIONE B: Modifica di un elemento esistente
            if dev_efforts_ref.row_count() > 0 {
                if let Some(mut effort) = dev_efforts_ref.row_data(0) {
                    effort.effort += 100; // Incrementa l'effort
                    dev_efforts_ref.set_row_data(0, effort); // ← Aggiornamento automatico!
                }

                if let Some(mut effort) = dev_efforts_ref.row_data(1) {
                    effort.effort += 1000; // Incrementa l'effort
                    dev_efforts_ref.set_row_data(1, effort); // ← Aggiornamento automatico!
                }
            }

            // OPZIONE C: Modifica delle date
            if dates_ref.row_count() < 60 {
                dates_ref.push(format!("25-12-{:02}", dates_ref.row_count()).into());
            }

            // OPZIONE E: Modifica modelli interni (worker_efforts, partial_efforts)
            worker_ref.push(format!("Timer Worker {}", worker_ref.row_count()).into());
            partial_ref.push(worker_ref.row_count() as i32 * 10);

            // OPZIONE D: Solo per proprietà non-model (come project_name) serve get/set
            if let Some(window) = main_window_weak.upgrade() {
                let mut data = window.get_pjm_datas();
                data.project_name =
                    format!("Aggiornato {}", Local::now().format("%H:%M:%S")).into();
                window.set_pjm_datas(data);
            }
        },
    );

    main_window.run().unwrap();
}

/*
RIEPILOGO METODI PER AGGIORNARE L'INTERFACCIA:

1. **MODIFICA DIRETTA (MIGLIORE)**:
   - Usa Rc<VecModel<T>> condivisi
   - model.push(), model.set_row_data(), model.remove()
   - Aggiornamento automatico dell'UI!

2. **GET/SET (MENO EFFICIENTE)**:
   - Solo per proprietà non-model
   - window.get_pjm_datas() → modifica → window.set_pjm_datas()

3. **ACCESSO AI MODELLI INTERNI**:
   - Mantieni riferimenti anche ai VecModel interni
   - worker_efforts, partial_efforts possono essere modificati direttamente

Il metodo get/set dovrebbe essere usato SOLO per proprietà semplici
come project_name, current_week, non per i VecModel!
*/
