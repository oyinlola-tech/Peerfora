fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("Peerfora administration application must start");
}
