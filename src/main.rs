mod config;
mod hyprland;
mod hyprlock;
mod state;
mod ui;
mod wallhaven;
use anyhow::Result;
use clap::{Parser, Subcommand};
use config::{expand_path, load_config};
use rand::Rng;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use wallhaven::{download_wallpaper, get_wallpaper_info, search_wallpapers};
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Subcommand)]
enum Commands {
    /// Rotate to a random wallpaper from the Hot list
    Rotate,
    /// Open the Fuzzel menu
    Menu,
    /// Search Wallhaven (opens browser)
    Search { query: String },
    /// Set a specific wallpaper by ID or URL
    Set { id_or_url: String },
    /// Init systemd units
    Init,
    /// Restore wallpapers from state
    Restore,
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = load_config()?;
    match cli.command {
        Some(Commands::Rotate) => {
            rotate_wallpaper(&mut config)?;
        }
        Some(Commands::Menu) => {
            handle_menu(&mut config)?;
        }
        Some(Commands::Search { query }) => {
            let url = format!("https://wallhaven.cc/search?q={}", query);
            open::that(url)?;
        }
        Some(Commands::Set { id_or_url }) => {
            // Extract ID if it's a URL
            let id = if id_or_url.contains("wallhaven.cc/w/") {
                id_or_url
                    .split("/w/")
                    .last()
                    .unwrap_or(&id_or_url)
                    .to_string()
            } else {
                id_or_url
            };
            set_specific_wallpaper(&id, &config)?;
        }
        Some(Commands::Init) => {
            println!("Please create a systemd timer to run 'wallhaven-cli rotate' periodically.");
        }
        Some(Commands::Restore) => {
            restore_wallpapers(&config)?;
        }
        None => {
            use clap::CommandFactory;
            Cli::command().print_help()?;
        }
    }
    Ok(())
}
fn rotate_wallpaper(config: &mut config::Config) -> Result<()> {
    // Get active monitor info
    let monitor = hyprland::get_active_monitor().unwrap_or_else(|e| {
        eprintln!(
            "Warning: Could not get active monitor: {}. Defaulting to config.",
            e
        );
        hyprland::Monitor {
            name: "".to_string(),
            width: 1920,
            height: 1080,
            focused: true,
            transform: 0,
            active_workspace: hyprland::ActiveWorkspace {
                id: 1,
            },
        }
    });
    let original_wallpaper = hyprland::get_current_wallpaper(&monitor.name).ok();
    let original_workspace_id = monitor.active_workspace.id;
    let (width, height) = monitor.get_visual_dimensions();
    let ratio = if width >= height {
        "landscape"
    } else {
        "portrait"
    };
    println!(
        "Detecting monitor: {} ({}) - Ratio: {}",
        monitor.name,
        if monitor.name.is_empty() {
            "Fallback"
        } else {
            "Active"
        },
        ratio
    );
    // 1. Search for wallpapers (Hot list)
    // We use a random page to get more variety
    let page = rand::thread_rng().gen_range(1..=3);
    let wallpapers = search_wallpapers(config, None, page, Some(ratio))?;
    if wallpapers.is_empty() {
        eprintln!("No wallpapers found.");
        return Ok(());
    }
    // Find empty workspace
    let occupied = hyprland::get_occupied_workspaces().unwrap_or_default();
    let mut empty_workspace_id = 10;
    while occupied.contains(&empty_workspace_id) {
        empty_workspace_id += 1;
    }
    // Switch to empty workspace
    hyprland::dispatch_workspace(empty_workspace_id)?;
    // 4. Interactive Loop
    // For rotate, we pick a random one initially.
    // We can iterate or re-roll. Let's stick to "Random" meaning "pick random from list".
    // "Next" meaning "next in list".
    let mut index = rand::thread_rng().gen_range(0..wallpapers.len());
    let total = wallpapers.len();
    let mut _current_set_path = None;
    // Set first immediately
    {
        let chosen_summary = &wallpapers[index];
        let chosen = get_wallpaper_info(&chosen_summary.id, config)?;
        let ext = chosen.path.split('.').last().unwrap_or("jpg");
        let filename = format!("wallhaven-{}.{}", chosen.id, ext);
        let save_path = expand_path(&config.save_dir).join(&filename);
        println!("Downloading {} to {:?}", chosen.id, save_path);
        download_wallpaper(&chosen.path, &save_path)?;
        set_system_wallpaper(&save_path, config, &monitor.name)?;
        _current_set_path = Some(save_path);
    }
    'nav_loop: loop {
        use ui::NavAction;
        match ui::show_search_nav_menu(index + 1, total)? {
            // index + 1 for display 1-based
            NavAction::Next => {
                index = (index + 1) % total;
            }
            NavAction::Prev => {
                if index == 0 {
                    index = total - 1;
                } else {
                    index -= 1;
                }
            }
            NavAction::Random => {
                index = rand::thread_rng().gen_range(0..total);
            }
            NavAction::OpenInBrowser => {
                hyprland::dispatch_workspace(original_workspace_id)?;
                let chosen_summary = &wallpapers[index];
                println!("Opening in browser: {}", chosen_summary.short_url);
                open::that(&chosen_summary.short_url)?;
                std::process::exit(0);
            }
            NavAction::Done => {
                hyprland::dispatch_workspace(original_workspace_id)?;
                std::process::exit(0);
            }
            NavAction::Cancel | NavAction::None => {
                hyprland::dispatch_workspace(original_workspace_id)?;
                if let Some(ref orig_path) = original_wallpaper {
                    println!("Restoring original wallpaper: {}", orig_path);
                    set_system_wallpaper(&PathBuf::from(orig_path), config, &monitor.name)?;
                }
                break 'nav_loop;
            }
        }
        // Apply new selection
        let chosen_summary = &wallpapers[index];
        let chosen = get_wallpaper_info(&chosen_summary.id, config)?;
        let ext = chosen.path.split('.').last().unwrap_or("jpg");
        let filename = format!("wallhaven-{}.{}", chosen.id, ext);
        let save_path = expand_path(&config.save_dir).join(&filename);
        download_wallpaper(&chosen.path, &save_path)?;
        set_system_wallpaper(&save_path, config, &monitor.name)?;
        _current_set_path = Some(save_path);
    }
    Ok(())
}
fn set_specific_wallpaper(id: &str, config: &config::Config) -> Result<()> {
    let wallpaper = get_wallpaper_info(id, config)?;
    // Get active monitor info
    let monitor = hyprland::get_active_monitor().unwrap_or_else(|_| hyprland::Monitor {
        name: "".to_string(),
        width: 1920,
        height: 1080,
        focused: true,
        transform: 0,
        active_workspace: hyprland::ActiveWorkspace {
            id: 1,
        },
    });
    let original_wallpaper = hyprland::get_current_wallpaper(&monitor.name).ok();
    let original_workspace_id = monitor.active_workspace.id;
    // Switch to empty workspace
    let occupied = hyprland::get_occupied_workspaces().unwrap_or_default();
    let mut empty_workspace_id = 10;
    while occupied.contains(&empty_workspace_id) {
        empty_workspace_id += 1;
    }
    hyprland::dispatch_workspace(empty_workspace_id)?;
    let ext = wallpaper.path.split('.').last().unwrap_or("jpg");
    let filename = format!("wallhaven-{}.{}", wallpaper.id, ext);
    let save_path = expand_path(&config.save_dir).join(&filename);
    println!("Downloading {}...", wallpaper.id);
    download_wallpaper(&wallpaper.path, &save_path)?;
    set_system_wallpaper(&save_path, config, &monitor.name)?;
    // Preview Loop
    loop {
        use ui::NavAction;
        match ui::show_preview_menu()? {
            NavAction::Done => {
                hyprland::dispatch_workspace(original_workspace_id)?;
                std::process::exit(0);
            }
            NavAction::Cancel => {
                hyprland::dispatch_workspace(original_workspace_id)?;
                if let Some(ref orig_path) = original_wallpaper {
                    println!("Restoring original wallpaper: {}", orig_path);
                    set_system_wallpaper(&PathBuf::from(orig_path), config, &monitor.name)?;
                }
                break;
            }
            NavAction::OpenInBrowser => {
                hyprland::dispatch_workspace(original_workspace_id)?;
                println!("Opening in browser: {}", wallpaper.short_url);
                open::that(&wallpaper.short_url)?;
                std::process::exit(0);
            }
            _ => {
                hyprland::dispatch_workspace(original_workspace_id)?;
                break;
            }
        }
    }
    Ok(())
}
fn set_system_wallpaper(path: &PathBuf, config: &config::Config, monitor_name: &str) -> Result<()> {
    let path_str = path.to_string_lossy();
    let mut cmd_str = config.wallpaper_cmd.replace("%f", &path_str);
    // Replace monitor placeholder
    cmd_str = cmd_str.replace("%m", monitor_name);
    // Also replace literal "monitor_name" just in case user has old config or literal instruction
    cmd_str = cmd_str.replace("monitor_name", monitor_name);
    println!("Executing: {}", cmd_str);
    
    // Execute command
    for sub_cmd in cmd_str.split(';') {
        let trimmed = sub_cmd.trim();
        if trimmed.is_empty() {
            continue;
        }
        let status = Command::new("sh").arg("-c").arg(trimmed).spawn()?.wait()?;
        if !status.success() {
            return Err(anyhow::anyhow!("Command failed: {}", trimmed));
        }
    }

    // Save State
    let mut state = state::load_state().unwrap_or_default();
    state.wallpapers.insert(monitor_name.to_string(), path_str.to_string());
    if let Err(e) = state::save_state(&state) {
        eprintln!("Warning: Failed to save state: {}", e);
    }

    // Update Hyprlock
    if let Err(e) = hyprlock::update_hyprlock_config(&state) {
        eprintln!("Warning: Failed to update hyprlock config: {}", e);
    }

    Ok(())
}

fn restore_wallpapers(config: &config::Config) -> Result<()> {
    let state = state::load_state()?;
    println!("Restoring wallpapers from state...");
    for (monitor, path) in &state.wallpapers {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            println!("Restoring {} on {}", path, monitor);
            // We reuse set_system_wallpaper but we must be careful not to create a loop
            // set_system_wallpaper saves state again. That's fine, it's idempotent.
            
            let mut attempts = 0;
            const MAX_ATTEMPTS: i32 = 5;
            loop {
                match set_system_wallpaper(&path_buf, config, monitor) {
                    Ok(_) => break,
                    Err(e) => {
                         attempts += 1;
                         if attempts >= MAX_ATTEMPTS {
                             eprintln!("Failed to restore wallpaper on {}: {} (after {} attempts)", monitor, e, attempts);
                             break;
                         }
                         eprintln!("Attempt {}/{} failed for {}: {}. Retrying in 1s...", attempts, MAX_ATTEMPTS, monitor, e);
                         thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        } else {
            eprintln!("Wallpaper not found: {}", path);
        }
    }
    Ok(())
}

fn handle_menu(config: &mut config::Config) -> Result<()> {
    loop {
        use ui::MenuAction;
        match ui::show_fuzzel_menu()? {
            MenuAction::Rotate => {
                rotate_wallpaper(config)?;
            }
            MenuAction::SearchApi => {
                search_interactive(config, None)?;
                // If search_interactive returns Ok(()), it means either Done or cancelled from query.
                // In either case, we want to stay in the main menu loop, not exit the app.
            }
            MenuAction::SetId => {
                let input = ui::get_user_input("Wallpaper ID/URL:")?;
                if !input.is_empty() {
                    // Extract ID if it's a URL
                    let id = if input.contains("wallhaven.cc/w/") {
                        input.split("/w/").last().unwrap_or(&input).to_string()
                    } else {
                        input
                    };
                    set_specific_wallpaper(&id, config)?;
                }
            }
            MenuAction::Settings => {
                handle_settings(config)?;
            }
            MenuAction::Custom(input) => {
                if input.is_empty() {
                    continue;
                }
                // Check 1: Wallhaven URL or ID
                if input.contains("wallhaven.cc/w/") {
                    let id = input.split("/w/").last().unwrap_or(&input).to_string();
                    set_specific_wallpaper(&id, config)?;
                    continue;
                }
                // Simple alphanumeric check for potential ID (length 6)
                if input.len() == 6 && input.chars().all(|c| c.is_alphanumeric()) {
                    // Could be an ID, try it? Or maybe this is too aggressive if user searches "flower" (6 chars).
                    // Wallhaven IDs are alphanumeric. "flower" is 6 chars.
                    // Let's be safe: Only treat as ID if it looks like an ID AND user explicitly picked "Set ID" menu option.
                    // But here we are in Custom input from main menu.
                    // Let's stick to explicit URL or rely on fallback search.
                    // Actually, the user requirement says: "Check first if this is a wallhaven URL or ID"
                    // If I type "8yx6gd", it should probably work.
                    // Let's try to set it. If it fails, we can catch the error?
                    // `set_specific_wallpaper` will fetch info. If 404, it errors.
                    // But we can't easily recover to search here without complex error handling.
                    // Let's assume if it's 6 random chars it might be a search query.
                    // So maybe only explicit URL for now, OR if the user really meant ID they would use "Set ID".
                    // BUT the requirement says "Check ... or ID".
                    // Let's try: if it matches ID pattern, try to fetch info. If success, set it. If fail, search.
                    // That requires `get_wallpaper_info` to return Result.
                    // For now, let's prioritize the explicit URL check and the image extension check.
                    // If the user types a raw ID, they might have to use the "Set ID" menu or we treat it as search.
                    // Use case: pasting an ID.
                    // Let's try to fetch it.
                    match get_wallpaper_info(&input, config) {
                        Ok(_) => {
                            set_specific_wallpaper(&input, config)?;
                            continue;
                        }
                        Err(_) => {
                            // Fallthrough to search
                        }
                    }
                }
                // Check 2: Direct Image URL
                let lower_input = input.to_lowercase();
                if input.starts_with("http")
                    && (lower_input.ends_with(".jpg")
                        || lower_input.ends_with(".jpeg")
                        || lower_input.ends_with(".png")
                        || lower_input.ends_with(".webp"))
                {
                    set_direct_wallpaper(&input, config)?;
                    continue;
                }
                // Check 3: Fallback Search
                search_interactive(config, Some(input))?;
                // If search_interactive returns Ok(()), it means either Done or cancelled from query.
                // In either case, we want to stay in the main menu loop, not exit the app.
            }
            MenuAction::None => {
                return Ok(());
            }
        }
    }
}
fn handle_settings(config: &mut config::Config) -> Result<()> {
    loop {
        use ui::SettingsAction;
        match ui::show_settings_menu()? {
            SettingsAction::Categories => {
                loop {
                    if let Some(new_cats) = ui::show_categories_menu(&config.categories)? {
                        config.categories = new_cats;
                        config::save_config(config)?;
                    } else {
                        break; // Back to settings menu
                    }
                }
            }
            SettingsAction::Purity => {
                loop {
                    if let Some(new_purity) = ui::show_purity_menu(&config.purity)? {
                        config.purity = new_purity;
                        config::save_config(config)?;
                    } else {
                        break; // Back to settings menu
                    }
                }
            }
            SettingsAction::Sorting => {
                if let Some(new_sorting) = ui::show_sorting_menu(&config.sorting)? {
                    config.sorting = new_sorting;
                    config::save_config(config)?;
                }
            }
            SettingsAction::SetApiKey => {
                if let Ok(key) = ui::get_password_input("Enter Wallhaven API Key:") {
                    if !key.is_empty() {
                        config.api_key = Some(key);
                        config::save_config(config)?;
                    }
                }
            }
            SettingsAction::Back | SettingsAction::None => {
                break;
            }
        }
    }
    Ok(())
}
fn search_interactive(
    config: &mut config::Config,
    mut initial_query: Option<String>,
) -> Result<()> {
    // 1. Get Monitor & Original State
    let monitor = hyprland::get_active_monitor().unwrap_or_else(|e| {
        eprintln!(
            "Warning: Could not get active monitor: {}. Defaulting to config.",
            e
        );
        hyprland::Monitor {
            name: "".to_string(),
            width: 1920,
            height: 1080,
            focused: true,
            transform: 0,
            active_workspace: hyprland::ActiveWorkspace {
                id: 1,
            },
        }
    });
    let original_wallpaper = hyprland::get_current_wallpaper(&monitor.name).ok(); // It's okay if we fail to get it
    let original_workspace_id = monitor.active_workspace.id;
    'query_input_loop: loop {
        // 2. Prompt Query (if not provided)
        let query = match initial_query.take() {
            // .take() consumes the Option value
            Some(q) => q,
            None => {
                let q = ui::get_user_input("Search Query:")?;
                if q.is_empty() {
                    // User cancelled the search query, go back to main menu
                    return Ok(());
                }
                q
            }
        };
        // 3. Fetch Results
        let (width, height) = monitor.get_visual_dimensions();
        let ratio = if width >= height {
            "landscape"
        } else {
            "portrait"
        };
        println!("Searching '{}' for {} ({})", query, monitor.name, ratio);
        let wallpapers = search_wallpapers(config, Some(&query), 1, Some(ratio))?;
        if wallpapers.is_empty() {
            eprintln!("No results found for '{}'.", query);
            continue 'query_input_loop; // Go back to query prompt
        }
        // Find empty workspace
        let occupied = hyprland::get_occupied_workspaces().unwrap_or_default();
        let mut empty_workspace_id = 10; // Start checking from 10 to preserve early workspaces
        while occupied.contains(&empty_workspace_id) {
            empty_workspace_id += 1;
        }
        // Switch to empty workspace
        hyprland::dispatch_workspace(empty_workspace_id)?;
        // 4. Interactive Loop
        let mut index = 0;
        let total = wallpapers.len();
        let mut _current_set_path = None;
        // Set first immediately
        {
            let chosen_summary = &wallpapers[index];
            // Fetch full details to get authorized download URL
            let chosen = get_wallpaper_info(&chosen_summary.id, config)?;
            let ext = chosen.path.split('.').last().unwrap_or("jpg");
            let filename = format!("wallhaven-{}.{}", chosen.id, ext);
            let save_path = expand_path(&config.save_dir).join(&filename);
            download_wallpaper(&chosen.path, &save_path)?;
            set_system_wallpaper(&save_path, config, &monitor.name)?;
            _current_set_path = Some(save_path);
        }
        'nav_loop: loop {
            use ui::NavAction;
            match ui::show_search_nav_menu(index, total)? {
                NavAction::Next => {
                    index = (index + 1) % total;
                }
                NavAction::Prev => {
                    if index == 0 {
                        index = total - 1;
                    } else {
                        index -= 1;
                    }
                }
                NavAction::Random => {
                    index = rand::thread_rng().gen_range(0..total);
                }
                NavAction::OpenInBrowser => {
                    // Restore workspace
                    hyprland::dispatch_workspace(original_workspace_id)?;
                    // Construct the full search URL including all parameters
                    let search_url = format!(
                    "https://wallhaven.cc/search?q={}&categories={}&purity={}&sorting={}&ratios={}",
                    query, config.categories, config.purity, config.sorting, ratio
                );
                    println!("Opening search results in browser: {}", search_url);
                    open::that(search_url)?;
                    std::process::exit(0);
                }
                NavAction::Done => {
                    // Restore workspace before exiting
                    hyprland::dispatch_workspace(original_workspace_id)?;
                    std::process::exit(0);
                }
                NavAction::Cancel => {
                    // Restore workspace first (so we see the restoration happening on original workspace? or hidden?)
                    // Probably restore workspace first so user is back in context.
                    hyprland::dispatch_workspace(original_workspace_id)?;
                    // Restore original and go back to query prompt
                    if let Some(ref orig_path) = original_wallpaper {
                        println!("Restoring original wallpaper: {}", orig_path);
                        set_system_wallpaper(&PathBuf::from(orig_path), config, &monitor.name)?;
                    }
                    break 'nav_loop; // Break inner loop, go to query_input_loop
                }
                NavAction::None => {
                    hyprland::dispatch_workspace(original_workspace_id)?;
                    // User escaped menu without explicit selection. Treat as cancel.
                    if let Some(ref orig_path) = original_wallpaper {
                        println!("Restoring original wallpaper: {}", orig_path);
                        set_system_wallpaper(&PathBuf::from(orig_path), config, &monitor.name)?;
                    }
                    break 'nav_loop; // Break inner loop, go to query_input_loop
                }
            }
            // Apply new selection (if not done/cancel)
            let chosen_summary = &wallpapers[index];
            let chosen = get_wallpaper_info(&chosen_summary.id, config)?;
            let ext = chosen.path.split('.').last().unwrap_or("jpg");
            let filename = format!("wallhaven-{}.{}", chosen.id, ext);
            let save_path = expand_path(&config.save_dir).join(&filename);
            // Only download if changed? (Always download for now, it checks existence inside)
            download_wallpaper(&chosen.path, &save_path)?;
            set_system_wallpaper(&save_path, config, &monitor.name)?;
            _current_set_path = Some(save_path);
        }
    }
}
fn set_direct_wallpaper(url: &str, config: &config::Config) -> Result<()> {
    // Try to get monitor
    let monitor = hyprland::get_active_monitor().unwrap_or_else(|_| hyprland::Monitor {
        name: "".to_string(),
        width: 1920,
        height: 1080,
        focused: true,
        transform: 0,
        active_workspace: hyprland::ActiveWorkspace {
            id: 1,
        },
    });
    let original_wallpaper = hyprland::get_current_wallpaper(&monitor.name).ok();
    let original_workspace_id = monitor.active_workspace.id;
    // Switch to empty workspace
    let occupied = hyprland::get_occupied_workspaces().unwrap_or_default();
    let mut empty_workspace_id = 10;
    while occupied.contains(&empty_workspace_id) {
        empty_workspace_id += 1;
    }
    hyprland::dispatch_workspace(empty_workspace_id)?;
    // Derive filename
    let filename = url.split('/').last().unwrap_or("wallpaper.jpg");
    let filename = if filename.is_empty() {
        "wallpaper.jpg"
    } else {
        filename
    };
    let save_path = expand_path(&config.save_dir).join(filename);
    println!("Downloading direct image to {:?}", save_path);
    download_wallpaper(url, &save_path)?;
    set_system_wallpaper(&save_path, config, &monitor.name)?;
    // Preview Loop
    loop {
        use ui::NavAction;
        match ui::show_preview_menu()? {
            NavAction::Done => {
                hyprland::dispatch_workspace(original_workspace_id)?;
                std::process::exit(0);
            }
            NavAction::Cancel => {
                hyprland::dispatch_workspace(original_workspace_id)?;
                if let Some(ref orig_path) = original_wallpaper {
                    println!("Restoring original wallpaper: {}", orig_path);
                    set_system_wallpaper(&PathBuf::from(orig_path), config, &monitor.name)?;
                }
                break;
            }
            NavAction::OpenInBrowser => {
                hyprland::dispatch_workspace(original_workspace_id)?;
                println!("Opening in browser: {}", url);
                open::that(url)?;
                std::process::exit(0);
            }
            _ => {
                hyprland::dispatch_workspace(original_workspace_id)?;
                break;
            }
        }
    }
    Ok(())
}
