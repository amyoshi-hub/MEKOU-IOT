use std::fs;
use url::Url;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri::AppHandle;
use tauri::{WebviewUrl};
use tauri::webview::WebviewWindowBuilder;

// world_list.json のエントリの構造を定義
#[derive(Debug, Serialize, Deserialize)]
pub struct WorldEntry {
    id: String,
    name: String,
    entry_point_path: String,
    // 必要に応じて、WASMパスやその他のメタデータも追加可能
}

// world_list.json ファイル全体の構造
#[derive(Debug, Serialize, Deserialize)]
pub struct WorldList {
    worlds: Vec<WorldEntry>,
}

fn unzip_file(zip_file_path: &Path, dest_dir: &Path) -> Result<(), String> {
    println!("Rust: Unzipping {} to {}", zip_file_path.display(), dest_dir.display());
    let file = fs::File::open(zip_file_path)
        .map_err(|e| {
            let err_msg = format!("Failed to open zip file {}: {}", zip_file_path.display(), e);
            println!("Error in unzip_file (open): {}", err_msg); // ここにログを追加
            err_msg
        })?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| {
            let err_msg = format!("Failed to create zip archive: {}", e);
            println!("Error in unzip_file (create archive): {}", err_msg); // ここにログを追加
            err_msg
        })?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| {
                let err_msg = format!("Failed to get file from zip archive at index {}: {}", i, e);
                println!("Error in unzip_file (get file from archive): {}", err_msg); // ここにログを追加
                err_msg
            })?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if (&*file.name()).ends_with('/') {
            // ディレクトリの場合
            fs::create_dir_all(&outpath)
                .map_err(|e| {
                    let err_msg = format!("Failed to create directory {}: {}", outpath.display(), e);
                    println!("Error in unzip_file (create dir): {}", err_msg); // ここにログを追加
                    err_msg
                })?;
        } else {
            // ファイルの場合
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)
                        .map_err(|e| {
                            let err_msg = format!("Failed to create parent directory {}: {}", p.display(), e);
                            println!("Error in unzip_file (create parent dir): {}", err_msg); // ここにログを追加
                            err_msg
                        })?;
                }
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| {
                    let err_msg = format!("Failed to create file {}: {}", outpath.display(), e);
                    println!("Error in unzip_file (create file): {}", err_msg); // ここにログを追加
                    err_msg
                })?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| {
                    let err_msg = format!("Failed to copy file content to {}: {}", outpath.display(), e);
                    println!("Error in unzip_file (copy content): {}", err_msg); // ここにログを追加
                    err_msg
                })?;
        }
        // Unixパーミッションをセット（オプション）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                    .map_err(|e| {
                        let err_msg = format!("Failed to set permissions for {}: {}", outpath.display(), e);
                        println!("Error in unzip_file (set permissions): {}", err_msg); // ここにログを追加
                        err_msg
                    })?;
            }
        }
    }
    println!("Rust: Unzipping successful for {}", zip_file_path.display()); // 成功ログ
    Ok(())
}

#[tauri::command]
pub async fn process_and_add_world(source_path: String, world_name: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    println!("Rust: Processing and adding world...");

    let source_path_buf = PathBuf::from(&source_path);
    let file_stem = source_path_buf.file_stem() // 拡張子なしのファイル名 (例: "my_game")
                                   .and_then(|s| s.to_str())
                                   .ok_or_else(|| "Invalid source path: missing file stem".to_string())?
                                   .to_string();
    let extension = source_path_buf.extension() // 拡張子 (例: "zip", "html", "wasm")
                                   .and_then(|s| s.to_str())
                                   .unwrap_or_default();

    // アプリケーションのデータディレクトリを取得
    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    println!("App Data Directory: {}", app_data_dir.display());

    let worlds_root_dir = app_data_dir.join("worlds");
    println!("Worlds Root Directory: {}", worlds_root_dir.display());

    let world_destination_dir = worlds_root_dir.join(&file_stem);

    // worlds ルートディレクトリが存在しない場合は作成
    if !worlds_root_dir.exists() {
        fs::create_dir_all(&worlds_root_dir)
            .map_err(|e| format!("Failed to create worlds root directory: {}", e))?;
    }

    let mut entry_point_relative_path = String::new(); // world_list.jsonに保存するパス

    if extension == "zip" {
        // ZIPファイルの場合、解凍
        if world_destination_dir.exists() {
             // 既存のディレクトリがある場合は削除またはエラーにする
            fs::remove_dir_all(&world_destination_dir)
                .map_err(|e| format!("Failed to remove existing world directory {}: {}", world_destination_dir.display(), e))?;
        }
        fs::create_dir_all(&world_destination_dir)
            .map_err(|e| format!("Failed to create world destination directory for zip: {}", e))?;

        unzip_file(&source_path_buf, &world_destination_dir)?;

        let html_file_name = "index.html"; // Godotのデフォルト
        let godot_html_path = world_destination_dir.join(html_file_name);
        if !godot_html_path.exists() {
            // index.html が見つからない場合はエラー、または別のデフォルトを探す
            return Err(format!("Unzipped content does not contain {}. Please ensure it's a valid Godot Web Export.", html_file_name));
        }
        entry_point_relative_path = PathBuf::from(file_stem).join(html_file_name).to_str().unwrap().to_string();

    } else {
        // ZIPファイル以外の場合（単一のHTML/WASMなど）は、world_destination_dirの下に直接コピー
        if world_destination_dir.exists() {
             // 既に同名のフォルダがある場合は削除またはエラーにする
            fs::remove_dir_all(&world_destination_dir)
                .map_err(|e| format!("Failed to remove existing world directory {}: {}", world_destination_dir.display(), e))?;
        }
        fs::create_dir_all(&world_destination_dir)
            .map_err(|e| format!("Failed to create world destination directory for file: {}", e))?;
        
        let dest_file_path = world_destination_dir.join(source_path_buf.file_name().unwrap());
        fs::copy(&source_path_buf, &dest_file_path)
            .map_err(|e| format!("Failed to copy file from {} to {}: {}", source_path, dest_file_path.display(), e))?;

        // この場合のエントリポイントはコピーしたファイル自体
        entry_point_relative_path = PathBuf::from(file_stem).join(source_path_buf.file_name().unwrap()).to_str().unwrap().to_string();
    }

    // world_list.json を読み込むか、新規作成する
    let world_list_path = worlds_root_dir.join("world_list.json");
    println!("World List JSON Path: {}", world_list_path.display());
    let mut world_list: WorldList = if world_list_path.exists() {
        let json_str = fs::read_to_string(&world_list_path)
            .map_err(|e| format!("Failed to read world_list.json: {}", e))?;
        serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse world_list.json: {}", e))?
    } else {
        WorldList { worlds: Vec::new() }
    };

    // 新しいワールドエントリを追加 (既存の同名ワールドは削除)
    world_list.worlds.retain(|w| w.name != world_name); // 同じ名前のワールドがあれば削除
    let new_world_id = uuid::Uuid::new_v4().to_string(); // ユニークなIDを生成
    let new_entry = WorldEntry {
        id: new_world_id,
        name: world_name.clone(), // フロントから渡された名前を使用
        entry_point_path: entry_point_relative_path.clone(),
    };
    world_list.worlds.push(new_entry);

    // 更新されたワールドリストをJSONとして保存
    let updated_json = serde_json::to_string_pretty(&world_list)
        .map_err(|e| format!("Failed to serialize updated world_list.json: {}", e))?;
    fs::write(&world_list_path, updated_json)
        .map_err(|e| format!("Failed to write updated world_list.json: {}", e))?;

    println!("Rust: World '{}' added successfully. Entry point: {}", world_name, entry_point_relative_path);
    Ok(entry_point_relative_path) // 成功したエントリポイントパスを返す
}

#[tauri::command]
pub async fn get_world_list(app_handle: tauri::AppHandle) -> Result<WorldList, String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    let worlds_root_dir = app_data_dir.join("worlds");
    let world_list_path = worlds_root_dir.join("world_list.json");

    if !world_list_path.exists() {
        // ファイルが存在しない場合は空のリストを返す
        println!("Rust: world_list.json not found, returning empty list.");
        return Ok(WorldList { worlds: Vec::new() });
    }

    let json_str = tokio::fs::read_to_string(&world_list_path) // 非同期読み込み
        .await // await を忘れずに
        .map_err(|e| format!("Failed to read world_list.json: {}", e))?;

    serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse world_list.json: {}", e))
}

#[tauri::command]
pub async fn open_world(app_handle: AppHandle, entry_point_path: String, world_name: String) -> Result<(), String> {
    println!("Rust: Attempting to open game: {} at path: {}", world_name, entry_point_path);

    // アプリのデータディレクトリを取得
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    
    // Godotゲームのルートディレクトリ（例: .../app_data/worlds/imgui）
    let worlds_root_dir = app_data_dir.join("worlds");
    let world_base_dir = worlds_root_dir.join(&world_name); // Assuming world_name is the directory name

    // エントriポイントの絶対パスを構築
    let file_name = PathBuf::from(&entry_point_path)
                                .file_name()
                                .ok_or_else(|| "Invalid entry_point_path: no file name provided".to_string())?
                                .to_owned();

    let load_url_path = world_base_dir.join(file_name);

    let url_string = format!("file://{}", load_url_path.to_string_lossy());
    let parsed_url: Url = url_string.parse() // Explicitly parse to `Url`
        .map_err(|e: url::ParseError| format!("Failed to parse URL: {}", e.to_string()))?;

    let window_url = WebviewUrl::External(parsed_url);

    let main_window_handle_clone = app_handle.get_webview_window("main").clone(); 
    if let Some(main_window) = &main_window_handle_clone {
        main_window.hide().map_err(|e| format!("Failed to hide main window: {}", e.to_string()))?;
    }

    let app_handle_for_close = app_handle.clone();
    let world_name_for_close = world_name.clone();

    // 新しいウィンドウを開くためのWebviewWindowBuilderを設定
    let game_window_builder = WebviewWindowBuilder::new(&app_handle, format!("game_{}", world_name), window_url)
        .title(format!("OSAI Browser - {}", world_name))
        .inner_size(800.0, 600.0) // **FIXED HERE**
        .resizable(true); // Ensure this is also `with_resizable` if not already

    // ウィンドウをビルドし、エラーハンドリング
    let game_window = game_window_builder
        .build()
        .map_err(|e| format!("Failed to create game window: {}", e.to_string()))?;

    // ウィンドウが閉じられた時のイベントハンドリングを設定
    // ここで、AppHandleのクローンを新しいウィンドウイベントのクロージャに移動させます
    let app_handle_for_close = app_handle.clone();
    let world_name_for_close = world_name.clone(); // world_nameもクロージャ内で使うのでクローン

    game_window.on_window_event(move |event| {
        // `api` を受け取るために `CloseRequestedEvent` をパターンマッチ
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            // デフォルトのウィンドウを閉じる動作を防ぐ
            //api.prevent_default();

            // 非同期ランタイムで新しいタスクをスポーン
            let app_handle_inner = app_handle_for_close.clone();
            let world_name_inner = world_name_for_close.clone();
            tauri::async_runtime::spawn(async move {
                // メインウィンドウを再表示し、フォーカスする
                if let Some(main_window) = app_handle_inner.get_webview_window("main") {
                    main_window.show().unwrap_or_else(|e| println!("Failed to show main window: {}", e));
                    main_window.set_focus().unwrap_or_else(|e| println!("Failed to focus main window: {}", e));
                }
                // ゲームウィンドウを閉じる
                if let Some(game_win) = app_handle_inner.get_webview_window(&format!("game_{}", world_name_inner)) {
                     game_win.close().unwrap_or_else(|e| println!("Failed to close game window: {}", e));
                }
            });
        }
    });

    Ok(())
}
