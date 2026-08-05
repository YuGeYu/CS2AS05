use std::time::Duration;

use tauri::AppHandle;

use crate::services::demo;

pub fn start(app: &AppHandle) {
    let logical = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(2);
    let worker_count = (logical / 2).clamp(1, 2);
    for index in 0..worker_count {
        let handle = app.clone();
        let worker_id = format!("core-{index}");
        std::thread::Builder::new()
            .name(format!("demo-{worker_id}"))
            .spawn(move || loop {
                match demo::process_next_job(&handle, &worker_id) {
                    Ok(true) => {}
                    Ok(false) | Err(_) => std::thread::sleep(Duration::from_millis(250)),
                }
            })
            .expect("failed to start Demo analysis worker");
    }
    let handle = app.clone();
    std::thread::Builder::new()
        .name("demo-spatial-0".into())
        .spawn(move || loop {
            match demo::process_next_spatial_job(&handle, "spatial-0") {
                Ok(true) => {}
                Ok(false) | Err(_) => std::thread::sleep(Duration::from_millis(400)),
            }
        })
        .expect("failed to start Demo spatial worker");
}
