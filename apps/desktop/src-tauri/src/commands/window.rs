/// Open a new window of the given type.
/// Supported types: "graph", "build", "compare"
#[tauri::command]
pub fn open_window(
    window_type: String,
    label_suffix: Option<String>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let suffix = label_suffix.unwrap_or_else(|| {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("{:x}", ts % 0xFFFF)
    });

    let label = format!("{}-{}", window_type, suffix);
    let title = match window_type.as_str() {
        "graph" => format!("SnapBuild — Graph ({})", suffix),
        "build" => format!("SnapBuild — Build Output ({})", suffix),
        "compare" => format!("SnapBuild — Compare ({})", suffix),
        _ => format!("SnapBuild — {}", suffix),
    };

    // Build the URL with a query param so the frontend knows its role
    let url = format!("index.html?window={}&id={}", window_type, suffix);

    tauri::WebviewWindowBuilder::new(&app, &label, tauri::WebviewUrl::App(url.into()))
        .title(&title)
        .inner_size(1000.0, 700.0)
        .min_inner_size(600.0, 400.0)
        .decorations(false)
        .center()
        .build()
        .map_err(|e| format!("Failed to create window: {}", e))?;

    Ok(())
}
