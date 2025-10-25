use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq)]
enum ConnectionType {
    None,
    Usb,
    Wireless,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Package {
    name: String,
    #[serde(default)]
    is_system: bool,
    #[serde(skip)]
    is_selected: bool,
}

struct AppState {
    packages: Vec<Package>,
    connection: ConnectionType,
    connected_device: String,
}

#[derive(Serialize, Deserialize)]
struct Backup {
    timestamp: String,
    packages: Vec<String>,
}

const CRITICAL_PACKAGES: &[&str] = &[
    "com.android.systemui",
    "com.android.settings",
    "com.android.phone",
    "com.android.providers.settings",
    "com.android.providers.contacts",
    "com.android.vending",
    "com.google.android.gms",
    "com.android.inputmethod.latin",
    "com.android.launcher3",
];

const COMMON_BLOATWARE: &[&str] = &[
    "com.facebook.katana",
    "com.facebook.system",
    "com.facebook.appmanager",
    "com.facebook.services",
    "com.netflix.mediaclient",
    "com.spotify.music",
    "com.linkedin.android",
    "com.microsoft.office.excel",
    "com.microsoft.office.word",
    "com.microsoft.office.powerpoint",
    "com.microsoft.skype.raider",
    "com.android.bips",
    "com.android.bookmarkprovider",
    "com.android.dreams.basic",
    "com.android.dreams.phototable",
    "com.android.egg",
    "com.android.printspooler",
    "com.google.android.apps.docs",
    "com.google.android.apps.maps",
    "com.google.android.apps.photos",
    "com.google.android.apps.tachyon",
    "com.google.android.music",
    "com.google.android.videos",
    "com.google.android.youtube",
    "com.samsung.android.game.gamehome",
    "com.samsung.android.game.gametools",
    "com.samsung.android.bixby.agent",
    "com.samsung.android.app.spage",
    "com.samsung.android.messaging",
];

fn main() {
    print_header();

    if !check_adb() {
        eprintln!("{}", "Error: ADB is not installed or not in PATH".red().bold());
        eprintln!("ADB is required to use this tool");
        
        if confirm_action("Would you like to install ADB now?") {
            install_adb();
            // check again after install
            if !check_adb() {
                eprintln!("{}", "ADB installation failed or not in PATH. Please install manually.".red());
                eprintln!("Download from: https://developer.android.com/tools/releases/platform-tools");
                return;
            }
        } else {
            eprintln!("Please install Android SDK Platform Tools manually");
            eprintln!("Download from: https://developer.android.com/tools/releases/platform-tools");
            return;
        }
    }

    println!("{}", "ADB found".green());

    let mut state = AppState::new();

    loop {
        display_main_menu(&state);

        match get_user_choice() {
            Ok(choice) => match choice {
                1 => wireless_debugging_menu(&mut state),
                2 => list_all_packages(&mut state),
                3 => load_bloatware_list(&mut state),
                4 => interactive_mode(&mut state),
                5 => remove_single_package(&mut state),
                6 => restore_package(&mut state),
                7 => search_packages(&mut state),
                8 => list_connected_devices(),
                9 => create_backup(&state),
                10 => show_device_info(&mut state),
                11 => {
                    println!("{}", "Exiting... Goodbye!".yellow());
                    break;
                }
                _ => println!("{}", "Invalid choice!".red()),
            },
            Err(_) => println!("{}", "Invalid input! Please enter a number.".red()),
        }
    }
}

impl AppState {
    fn new() -> Self {
        AppState {
            packages: Vec::new(),
            connection: ConnectionType::None,
            connected_device: String::new(),
        }
    }
}

fn print_header() {
    println!("{}", "===========================================".cyan().bold());
    println!("{}", "        Android Debloater Tool            ".cyan().bold());
    println!("{}", "===========================================".cyan().bold());
    println!();
}

fn display_main_menu(state: &AppState) {
    println!();
    println!("{}", "===========================================".cyan());
    println!("{}", "              Main Menu                    ".cyan());
    println!("{}", "===========================================".cyan());

    match state.connection {
        ConnectionType::Wireless => {
            println!("{}", format!(" Status: {} {}", 
                "Wireless Connected".green(),
                " ".repeat(18)).cyan());
        }
        ConnectionType::Usb => {
            println!("{}", format!(" Status: {} {}", 
                "USB Connected".blue(),
                " ".repeat(23)).cyan());
        }
        ConnectionType::None => {
            println!("{}", format!(" Status: {} {}", 
                "Not Connected".red(),
                " ".repeat(23)).cyan());
        }
    }

    println!("{}", "===========================================".cyan());
    println!("{}", "  1. Wireless debugging menu              ".cyan());
    println!("{}", "  2. List all system packages             ".cyan());
    println!("{}", "  3. List common bloatware                ".cyan());
    println!("{}", "  4. Interactive removal mode             ".cyan());
    println!("{}", "  5. Remove specific package              ".cyan());
    println!("{}", "  6. Restore package                      ".cyan());
    println!("{}", "  7. Search packages                      ".cyan());
    println!("{}", "  8. Show connected devices               ".cyan());
    println!("{}", "  9. Create backup                        ".cyan());
    println!("{}", " 10. Show device info                     ".cyan());
    println!("{}", " 11. Exit                                 ".cyan());
    println!("{}", "===========================================".cyan());
}

fn display_wireless_menu() {
    println!();
    println!("{}", "===========================================".magenta());
    println!("{}", "       Wireless Debugging Menu            ".magenta());
    println!("{}", "===========================================".magenta());
    println!("{}", "  1. Pair & Connect (Android 11+)         ".magenta());
    println!("{}", "  2. Legacy wireless connection           ".magenta());
    println!("{}", "  3. Auto-detect device IP                ".magenta());
    println!("{}", "  4. Enable wireless on USB device        ".magenta());
    println!("{}", "  5. Disconnect wireless                  ".magenta());
    println!("{}", "  6. List connected devices               ".magenta());
    println!("{}", "  7. Back to main menu                    ".magenta());
    println!("{}", "===========================================".magenta());
}

fn get_user_choice() -> Result<usize, std::num::ParseIntError> {
    print!("{}", "Enter choice: ".bright_white().bold());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().parse()
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt.bright_white());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn confirm_action(prompt: &str) -> bool {
    let response = get_user_input(&format!("{} (yes/no): ", prompt));
    response.to_lowercase() == "yes" || response.to_lowercase() == "y"
}

fn check_adb() -> bool {
    Command::new("adb")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// try to install adb based on os
fn install_adb() {
    println!("{}", "Detecting your system...".yellow());
    
    let os = std::env::consts::OS;
    
    match os {
        "linux" => install_adb_linux(),
        "windows" => install_adb_windows(),
        "macos" => install_adb_macos(),
        _ => {
            println!("{}", "Unsupported OS. Please install ADB manually.".red());
        }
    }
}

fn install_adb_linux() {
    // try to detect which package manager to use
    println!("{}", "Detecting package manager...".yellow());
    
    // check for nix first
    if Command::new("nix-env").arg("--version").output().is_ok() {
        println!("{}", "Detected Nix package manager".green());
        if confirm_action("Install ADB using nix-env?") {
            println!("{}", "Installing android-tools...".yellow());
            let status = Command::new("nix-env")
                .args(&["-iA", "nixpkgs.android-tools"])
                .status();
            
            match status {
                Ok(s) if s.success() => println!("{}", "ADB installed successfully!".green()),
                _ => println!("{}", "Installation failed. Try manually with: nix-env -iA nixpkgs.android-tools".red()),
            }
        }
        return;
    }
    
    // check for pacman (arch)
    if Command::new("pacman").arg("--version").output().is_ok() {
        println!("{}", "Detected pacman (Arch Linux)".green());
        if confirm_action("Install ADB using pacman?") {
            println!("{}", "Installing android-tools...".yellow());
            println!("{}", "This requires sudo access".yellow());
            let status = Command::new("sudo")
                .args(&["pacman", "-S", "--noconfirm", "android-tools"])
                .status();
            
            match status {
                Ok(s) if s.success() => println!("{}", "ADB installed successfully!".green()),
                _ => println!("{}", "Installation failed. Try manually with: sudo pacman -S android-tools".red()),
            }
        }
        return;
    }
    
    // check for apt (debian/ubuntu)
    if Command::new("apt").arg("--version").output().is_ok() {
        println!("{}", "Detected APT (Debian/Ubuntu)".green());
        if confirm_action("Install ADB using apt?") {
            println!("{}", "Installing adb...".yellow());
            println!("{}", "This requires sudo access".yellow());
            let update = Command::new("sudo")
                .args(&["apt", "update"])
                .status();
                
            if update.is_ok() {
                let status = Command::new("sudo")
                    .args(&["apt", "install", "-y", "adb"])
                    .status();
                
                match status {
                    Ok(s) if s.success() => println!("{}", "ADB installed successfully!".green()),
                    _ => println!("{}", "Installation failed. Try manually with: sudo apt install adb".red()),
                }
            }
        }
        return;
    }
    
    // check for dnf (fedora)
    if Command::new("dnf").arg("--version").output().is_ok() {
        println!("{}", "Detected DNF (Fedora)".green());
        if confirm_action("Install ADB using dnf?") {
            println!("{}", "Installing android-tools...".yellow());
            println!("{}", "This requires sudo access".yellow());
            let status = Command::new("sudo")
                .args(&["dnf", "install", "-y", "android-tools"])
                .status();
            
            match status {
                Ok(s) if s.success() => println!("{}", "ADB installed successfully!".green()),
                _ => println!("{}", "Installation failed. Try manually with: sudo dnf install android-tools".red()),
            }
        }
        return;
    }
    
    // check for zypper (opensuse)
    if Command::new("zypper").arg("--version").output().is_ok() {
        println!("{}", "Detected Zypper (openSUSE)".green());
        if confirm_action("Install ADB using zypper?") {
            println!("{}", "Installing android-tools...".yellow());
            println!("{}", "This requires sudo access".yellow());
            let status = Command::new("sudo")
                .args(&["zypper", "install", "-y", "android-tools"])
                .status();
            
            match status {
                Ok(s) if s.success() => println!("{}", "ADB installed successfully!".green()),
                _ => println!("{}", "Installation failed. Try manually with: sudo zypper install android-tools".red()),
            }
        }
        return;
    }
    
    println!("{}", "Could not detect package manager".red());
    println!("{}", "Please install ADB manually for your distribution".yellow());
}

fn install_adb_windows() {
    println!("{}", "Detected Windows".green());
    
    // try winget first
    if Command::new("winget").arg("--version").output().is_ok() {
        println!("{}", "Found winget package manager".green());
        if confirm_action("Install ADB using winget?") {
            println!("{}", "Installing Android SDK Platform Tools...".yellow());
            let status = Command::new("winget")
                .args(&["install", "Google.PlatformTools"])
                .status();
            
            match status {
                Ok(s) if s.success() => {
                    println!("{}", "ADB installed successfully!".green());
                    println!("{}", "You may need to restart your terminal or add ADB to PATH".yellow());
                    return;
                }
                _ => println!("{}", "Installation with winget failed, trying chocolatey...".yellow()),
            }
        }
    }
    
    // try chocolatey
    if Command::new("choco").arg("--version").output().is_ok() {
        println!("{}", "Found Chocolatey package manager".green());
        if confirm_action("Install ADB using Chocolatey?") {
            println!("{}", "Installing adb...".yellow());
            let status = Command::new("choco")
                .args(&["install", "adb", "-y"])
                .status();
            
            match status {
                Ok(s) if s.success() => {
                    println!("{}", "ADB installed successfully!".green());
                    println!("{}", "You may need to restart your terminal".yellow());
                    return;
                }
                _ => println!("{}", "Installation failed".red()),
            }
        }
    }
    
    println!("{}", "No package manager found (winget or chocolatey)".red());
    println!("{}", "Please download ADB manually from:".yellow());
    println!("https://developer.android.com/tools/releases/platform-tools");
    println!("{}", "Extract and add to your PATH".yellow());
}

fn install_adb_macos() {
    println!("{}", "Detected macOS".green());
    
    // check for homebrew
    if Command::new("brew").arg("--version").output().is_ok() {
        println!("{}", "Found Homebrew".green());
        if confirm_action("Install ADB using Homebrew?") {
            println!("{}", "Installing android-platform-tools...".yellow());
            let status = Command::new("brew")
                .args(&["install", "android-platform-tools"])
                .status();
            
            match status {
                Ok(s) if s.success() => println!("{}", "ADB installed successfully!".green()),
                _ => println!("{}", "Installation failed. Try manually with: brew install android-platform-tools".red()),
            }
        }
        return;
    }
    
    println!("{}", "Homebrew not found".red());
    println!("{}", "Please install Homebrew first: https://brew.sh".yellow());
    println!("{}", "Or download ADB manually from:".yellow());
    println!("https://developer.android.com/tools/releases/platform-tools");
}

fn check_device_connected(state: &mut AppState) -> bool {
    let output = match Command::new("adb").arg("devices").output() {
        Ok(out) => out,
        Err(_) => return false,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    for (i, line) in stdout.lines().enumerate() {
        if i > 0 && line.contains("\tdevice") && !line.contains("List of") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                state.connected_device = parts[0].to_string();
                state.connection = if parts[0].contains(':') {
                    ConnectionType::Wireless
                } else {
                    ConnectionType::Usb
                };
                return true;
            }
        }
    }

    state.connection = ConnectionType::None;
    state.connected_device.clear();
    false
}

fn execute_adb_command(args: &[&str]) -> Result<String, String> {
    let output = Command::new("adb")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute ADB: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}


fn wireless_debugging_menu(state: &mut AppState) {
    loop {
        display_wireless_menu();

        match get_user_choice() {
            Ok(choice) => match choice {
                1 => connect_wireless_pairing(state),
                2 => connect_wireless_legacy(state),
                3 => get_device_ip_automatically(state),
                4 => enable_wireless_adb_on_device(state),
                5 => disconnect_wireless(state),
                6 => list_connected_devices(),
                7 => return,
                _ => println!("{}", "Invalid choice!".red()),
            },
            Err(_) => println!("{}", "Invalid input!".red()),
        }
    }
}

fn connect_wireless_pairing(state: &mut AppState) {
    println!();
    println!("{}", "=========================================".cyan());
    println!("{}", "  Wireless Pairing (Android 11+)".cyan().bold());
    println!("{}", "=========================================".cyan());

    println!("\n{}", "On your Android device:".yellow());
    println!("1. Go to: {} -> {}", "Settings".bright_white(), "Developer Options".bright_white());
    println!("2. Enable '{}'", "Wireless Debugging".bright_white());
    println!("3. Tap '{}'", "Pair device with pairing code".bright_white());
    println!("4. Note the IP address, port, and pairing code\n");

    let ip = get_user_input("Enter device IP address: ");
    if ip.is_empty() {
        println!("{}", "Operation cancelled.".yellow());
        return;
    }

    let port = get_user_input("Enter pairing port: ");
    let pairing_code = get_user_input("Enter pairing code: ");

    println!("{}", "\nPairing with device...".yellow());

    let result = Command::new("adb")
        .arg("pair")
        .arg(format!("{}:{}", ip, port))
        .arg(&pairing_code)
        .status();

    match result {
        Ok(status) if status.success() => {
            println!("{}", "Pairing successful!".green().bold());

            let conn_port = get_user_input("\nEnter connection port (usually 5555 or shown on device): ");

            println!("{}", "Connecting...".yellow());
            let connect_result = Command::new("adb")
                .arg("connect")
                .arg(format!("{}:{}", ip, conn_port))
                .status();

            if let Ok(status) = connect_result {
                if status.success() {
                    thread::sleep(Duration::from_millis(500));
                    println!("{}", "Connected wirelessly!".green().bold());
                    state.connection = ConnectionType::Wireless;
                    state.connected_device = format!("{}:{}", ip, conn_port);
                } else {
                    println!("{}", "Connection failed".red());
                }
            }
        }
        _ => {
            println!("{}", "Pairing failed".red().bold());
            println!("\n{}", "Troubleshooting:".yellow());
            println!("  - IP address and port are correct");
            println!("  - Pairing code is correct (6 digits)");
            println!("  - Wireless debugging is enabled on device");
            println!("  - Device and computer are on the same network");
        }
    }
}

fn connect_wireless_legacy(state: &mut AppState) {
    println!();
    println!("{}", "=========================================".cyan());
    println!("{}", "  Legacy Wireless Connection".cyan().bold());
    println!("{}", "=========================================".cyan());

    println!("\n{}", "Requirements:".yellow());
    println!("1. Device connected via USB first");
    println!("2. Device and computer on same Wi-Fi network\n");

    let (ip, port) = if !check_device_connected(state) || state.connection != ConnectionType::Usb {
        println!("{}", "No USB device detected. Manual connection mode.".yellow());

        let ip = get_user_input("\nEnter device IP address: ");
        let port = get_user_input("Enter port (default 5555, press Enter): ");
        let port = if port.is_empty() { "5555".to_string() } else { port };

        (ip, port)
    } else {
        println!("{}", "USB device detected!".green());

        println!("{}", "Enabling wireless debugging on device...".yellow());
        let _ = Command::new("adb")
            .args(&["tcpip", "5555"])
            .status();

        thread::sleep(Duration::from_secs(2));

        println!("{}", "Detecting device IP address...".yellow());

        let ip = match execute_adb_command(&["shell", "ip", "addr", "show", "wlan0"]) {
            Ok(output) => extract_ip_from_output(&output)
                .unwrap_or_else(|| get_user_input("Could not auto-detect IP. Enter device IP address: ")),
            Err(_) => get_user_input("Enter device IP address: "),
        };

        if !ip.is_empty() && !ip.contains("Enter") {
            println!("{} {}", "Device IP:".green(), ip.bright_white());
        }

        (ip, "5555".to_string())
    };

    if ip.is_empty() {
        println!("{}", "Operation cancelled.".yellow());
        return;
    }

    println!("{} {}:{}...", "Connecting to".yellow(), ip.bright_white(), port.bright_white());

    let result = Command::new("adb")
        .arg("connect")
        .arg(format!("{}:{}", ip, port))
        .status();

    if let Ok(status) = result {
        if status.success() {
            thread::sleep(Duration::from_secs(1));
            if check_device_connected(state) {
                println!("{} {}:{}", "Connected wirelessly to".green().bold(), ip, port);
                state.connection = ConnectionType::Wireless;
                state.connected_device = format!("{}:{}", ip, port);
                return;
            }
        }
    }

    println!("{}", "Connection failed".red().bold());
    println!("\n{}", "Troubleshooting:".yellow());
    println!("  - Ensure device and computer are on same Wi-Fi");
    println!("  - Check if wireless debugging is enabled");
    println!("  - Verify the IP address is correct");
    println!("  - Try: adb kill-server && adb start-server");
}

fn extract_ip_from_output(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("inet ") && !line.contains("inet6") && !line.contains("127.0.0.1") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "inet" && i + 1 < parts.len() {
                    if let Some(ip) = parts[i + 1].split('/').next() {
                        return Some(ip.to_string());
                    }
                }
            }
        }
    }
    None
}

fn get_device_ip_automatically(state: &mut AppState) {
    println!();
    println!("{}", "=========================================".cyan());
    println!("{}", "  Auto-detect Device IP".cyan().bold());
    println!("{}", "=========================================".cyan());

    if !check_device_connected(state) {
        println!("{}", "No device connected via USB!".red());
        println!("Please connect device via USB first.");
        return;
    }

    println!("{}", "Detecting IP address...".yellow());

    match execute_adb_command(&["shell", "ip", "addr", "show", "wlan0"]) {
        Ok(output) => {
            if let Some(ip) = extract_ip_from_output(&output) {
                println!("{} {}", "Device IP Address:".green().bold(), ip.bright_white());
                println!("{}", "You can now use this IP to connect wirelessly.".cyan());
            } else {
                println!("{}", "Could not detect IP address".red());
                print_ip_troubleshooting();
            }
        }
        Err(_) => {
            println!("{}", "Could not detect IP address".red());
            print_ip_troubleshooting();
        }
    }
}

fn print_ip_troubleshooting() {
    println!("\n{}", "Make sure:".yellow());
    println!("  - Device is connected to Wi-Fi");
    println!("  - Device is connected via USB");
    println!("  - USB debugging is enabled");
}

fn enable_wireless_adb_on_device(state: &mut AppState) {
    println!();
    println!("{}", "=========================================".cyan());
    println!("{}", "  Enable Wireless Debugging".cyan().bold());
    println!("{}", "=========================================".cyan());

    if !check_device_connected(state) || state.connection != ConnectionType::Usb {
        println!("{}", "Please connect device via USB first!".red());
        return;
    }

    println!("{}", "Enabling TCP/IP mode on port 5555...".yellow());

    let result = Command::new("adb")
        .args(&["tcpip", "5555"])
        .status();

    match result {
        Ok(status) if status.success() => {
            println!("{}", "Wireless debugging enabled!".green().bold());
            println!("\n{}", "Next steps:".cyan());
            println!("  1. Disconnect USB cable (optional)");
            println!("  2. Use option 3 to get device IP");
            println!("  3. Use option 2 to connect wirelessly");
        }
        _ => {
            println!("{}", "Failed to enable wireless debugging".red());
        }
    }
}

fn disconnect_wireless(state: &mut AppState) {
    if state.connection == ConnectionType::Wireless {
        println!("{} {}...", "Disconnecting from".yellow(), state.connected_device);

        let _ = Command::new("adb")
            .arg("disconnect")
            .arg(&state.connected_device)
            .status();

        state.connection = ConnectionType::None;
        state.connected_device.clear();

        println!("{}", "Disconnected".green());
    } else {
        println!("{}", "No wireless connection active".yellow());
    }
}

fn list_connected_devices() {
    println!();
    println!("{}", "=========================================".cyan());
    println!("{}", "  Connected Devices".cyan().bold());
    println!("{}", "=========================================".cyan());

    match execute_adb_command(&["devices", "-l"]) {
        Ok(output) => {
            let mut count = 0;

            for (i, line) in output.lines().enumerate() {
                if i > 0 && line.contains("\tdevice") && !line.contains("List of") {
                    count += 1;

                    if line.contains(':') {
                        print!("{} ", "[Wireless]".green().bold());
                    } else {
                        print!("{} ", "[USB]     ".blue().bold());
                    }

                    println!("{}", line.bright_white());
                }
            }

            println!("{}", "=========================================".cyan());
            if count == 0 {
                println!("{}", "No devices connected".yellow());
            } else {
                println!("{} {}", "Total:".cyan(), format!("{} device(s)", count).bright_white());
            }
        }
        Err(e) => {
            println!("{} {}", "Error:".red(), e);
        }
    }
}


fn list_all_packages(state: &mut AppState) {
    if !check_device_connected(state) {
        println!("{}", "Error: No device connected!".red());
        println!("Please connect a device first (USB or Wireless)");
        return;
    }

    println!("{}", "Fetching all packages from device...".yellow());

    match execute_adb_command(&["shell", "pm", "list", "packages", "-s"]) {
        Ok(output) => {
            state.packages.clear();

            for line in output.lines() {
                if let Some(package_name) = line.strip_prefix("package:") {
                    state.packages.push(Package {
                        name: package_name.to_string(),
                        is_system: true,
                        is_selected: false,
                    });
                }
            }

            state.packages.sort_by(|a, b| a.name.cmp(&b.name));

            println!("{} {} system packages", 
                "Found".green().bold(), 
                state.packages.len().to_string().bright_white()
            );
            display_packages(&state.packages, None);
        }
        Err(e) => {
            println!("{} {}", "Error executing ADB command:".red(), e);
        }
    }
}

fn load_bloatware_list(state: &mut AppState) {
    if !check_device_connected(state) {
        println!("{}", "Error: No device connected!".red());
        return;
    }

    println!("{}", "Scanning for common bloatware packages...".yellow());

    state.packages.clear();
    let mut found_packages = HashSet::new();

    for bloatware in COMMON_BLOATWARE {
        match execute_adb_command(&["shell", "pm", "list", "packages", bloatware]) {
            Ok(output) => {
                if output.contains(bloatware) {
                    if found_packages.insert(bloatware.to_string()) {
                        state.packages.push(Package {
                            name: bloatware.to_string(),
                            is_system: true,
                            is_selected: false,
                        });
                    }
                }
            }
            Err(_) => {}
        }
    }

    state.packages.sort_by(|a, b| a.name.cmp(&b.name));

    println!("{} {} bloatware packages installed", 
        "Found".green().bold(), 
        state.packages.len().to_string().bright_white()
    );
    
    if state.packages.is_empty() {
        println!("{}", "Great! No common bloatware detected.".green());
    } else {
        display_packages(&state.packages, None);
    }
}

fn display_packages(packages: &[Package], filter: Option<&str>) {
    println!();
    println!("{}", "===========================================".blue());
    println!("{}", "           Package List".blue().bold());
    println!("{}", "===========================================".blue());

    let mut displayed = 0;
    for (i, package) in packages.iter().enumerate() {
        if filter.is_none() || package.name.to_lowercase().contains(&filter.unwrap().to_lowercase()) {
            let checkbox = if package.is_selected { "[X]".green().bold() } else { "[ ]".normal() };
            let number = format!("[{:3}]", i + 1).bright_black();
            
            let package_display = if is_critical_package(&package.name) {
                format!("{} (CRITICAL)", package.name).red()
            } else {
                package.name.normal()
            };
            
            println!("{} {} {}", checkbox, number, package_display);
            displayed += 1;
        }
    }

    println!("{}", "===========================================".blue());
    println!("{} {}", "Displayed:".cyan(), format!("{} packages", displayed).bright_white());
    println!();
}

fn interactive_mode(state: &mut AppState) {
    if !check_device_connected(state) {
        println!("{}", "Error: No device connected!".red());
        return;
    }

    if state.packages.is_empty() {
        println!("{}", "Please load packages first (option 2 or 3)".yellow());
        return;
    }

    loop {
        display_packages(&state.packages, None);

        println!("{}", "Interactive Mode:".yellow().bold());
        println!("Enter package number to toggle selection, or:");
        let count = state.packages.len();
        println!("  {} - Select all", (count + 1).to_string().bright_white());
        println!("  {} - Deselect all", (count + 2).to_string().bright_white());
        println!("  {} - Filter/Search", (count + 3).to_string().bright_white());
        println!("  {} - Remove selected packages", (count + 4).to_string().bright_white());
        println!("  {} - Back to main menu", (count + 5).to_string().bright_white());

        match get_user_choice() {
            Ok(choice) => {
                if choice >= 1 && choice <= count {
                    state.packages[choice - 1].is_selected = !state.packages[choice - 1].is_selected;
                    let status = if state.packages[choice - 1].is_selected { "Selected" } else { "Deselected" };
                    println!("{} {}", 
                        format!("{}:", status).green(), 
                        state.packages[choice - 1].name.bright_white()
                    );
                    thread::sleep(Duration::from_millis(300));
                } else if choice == count + 1 {
                    for package in &mut state.packages {
                        package.is_selected = true;
                    }
                    println!("{}", "All packages selected".green());
                    thread::sleep(Duration::from_millis(500));
                } else if choice == count + 2 {
                    for package in &mut state.packages {
                        package.is_selected = false;
                    }
                    println!("{}", "All packages deselected".green());
                    thread::sleep(Duration::from_millis(500));
                } else if choice == count + 3 {
                    let filter = get_user_input("Enter search term: ");
                    display_packages(&state.packages, Some(&filter));
                    println!("{}", "Press Enter to continue...".bright_black());
                    let _ = get_user_input("");
                } else if choice == count + 4 {
                    let selected: Vec<_> = state.packages.iter()
                        .filter(|p| p.is_selected)
                        .collect();

                    if selected.is_empty() {
                        println!("{}", "No packages selected!".yellow());
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }

                    let critical_selected: Vec<_> = selected.iter()
                        .filter(|p| is_critical_package(&p.name))
                        .collect();

                    if !critical_selected.is_empty() {
                        println!("{}", "WARNING: Critical system packages selected!".red().bold());
                        for pkg in &critical_selected {
                            println!("  - {}", pkg.name.red());
                        }
                        if !confirm_action("This may cause system instability. Continue?") {
                            continue;
                        }
                    }

                    if confirm_action(&format!("Remove {} packages?", selected.len())) {
                        println!();
                        for package in selected {
                            remove_package(&package.name);
                        }
                        println!("{}", "\nOperation completed!".green().bold());
                        thread::sleep(Duration::from_secs(2));
                    }
                } else if choice == count + 5 {
                    break;
                }
            }
            Err(_) => println!("{}", "Invalid input!".red()),
        }
    }
}

fn remove_single_package(state: &mut AppState) {
    if !check_device_connected(state) {
        println!("{}", "Error: No device connected!".red());
        return;
    }

    let package_name = get_user_input("Enter package name to remove: ");

    if package_name.is_empty() {
        println!("{}", "Operation cancelled.".yellow());
        return;
    }

    if is_critical_package(&package_name) {
        println!("{}", "WARNING: This is a critical system package!".red().bold());
        if !confirm_action("Removing it may cause system instability. Continue?") {
            println!("Cancelled.");
            return;
        }
    }

    remove_package(&package_name);
}

fn remove_package(package_name: &str) {
    println!("{} {}", "Removing package:".yellow(), package_name.bright_white());

    let result = Command::new("adb")
        .args(&["shell", "pm", "uninstall", "--user", "0", package_name])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            println!("{} {}", "Successfully removed:".green().bold(), package_name);
        }
        _ => {
            println!("{} {}", "Failed to remove:".red(), package_name);

            println!("{}", "Trying to disable instead...".yellow());

            let disable_result = Command::new("adb")
                .args(&["shell", "pm", "disable-user", "--user", "0", package_name])
                .output();

            match disable_result {
                Ok(output) if output.status.success() => {
                    println!("{} {}", "Successfully disabled:".green().bold(), package_name);
                }
                _ => {
                    println!("{} {}", "Failed to disable:".red(), package_name);
                }
            }
        }
    }
}

fn restore_package(state: &mut AppState) {
    if !check_device_connected(state) {
        println!("{}", "Error: No device connected!".red());
        return;
    }

    let package_name = get_user_input("Enter package name to restore: ");

    if package_name.is_empty() {
        println!("{}", "Operation cancelled.".yellow());
        return;
    }

    println!("{} {}", "Restoring package:".yellow(), package_name.bright_white());

    let result = Command::new("adb")
        .args(&["shell", "cmd", "package", "install-existing", package_name.as_str()])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            println!("{} {}", "Successfully restored:".green().bold(), package_name);
        }
        _ => {
            println!("{} {}", "Failed to restore:".red(), package_name);
            println!("The package may not have been previously installed on this device.");
        }
    }
}

fn search_packages(state: &mut AppState) {
    if !check_device_connected(state) {
        println!("{}", "Error: No device connected!".red());
        return;
    }

    let search_term = get_user_input("Enter search term: ");

    if search_term.is_empty() {
        return;
    }

    println!("{} '{}'...", "Searching for".yellow(), search_term.bright_white());

    match execute_adb_command(&["shell", "pm", "list", "packages"]) {
        Ok(output) => {
            let mut found_packages = Vec::new();

            for line in output.lines() {
                if let Some(package_name) = line.strip_prefix("package:") {
                    if package_name.to_lowercase().contains(&search_term.to_lowercase()) {
                        found_packages.push(Package {
                            name: package_name.to_string(),
                            is_system: true,
                            is_selected: false,
                        });
                    }
                }
            }

            if found_packages.is_empty() {
                println!("{}", "No packages found matching search term.".yellow());
            } else {
                println!("{} {} packages", 
                    "Found".green().bold(), 
                    found_packages.len().to_string().bright_white()
                );
                display_packages(&found_packages, None);
            }
        }
        Err(e) => {
            println!("{} {}", "Error:".red(), e);
        }
    }
}

fn is_critical_package(package_name: &str) -> bool {
    CRITICAL_PACKAGES.contains(&package_name)
}

fn create_backup(state: &AppState) {
    if state.packages.is_empty() {
        println!("{}", "No packages loaded. Load packages first.".yellow());
        return;
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("backup_{}.json", timestamp);

    let backup = Backup {
        timestamp: timestamp.clone(),
        packages: state.packages.iter().map(|p| p.name.clone()).collect(),
    };

    match serde_json::to_string_pretty(&backup) {
        Ok(json) => {
            if let Err(e) = fs::write(&filename, json) {
                println!("{} {}", "Failed to create backup:".red(), e);
            } else {
                println!("{} {}", "Backup created:".green().bold(), filename.bright_white());
            }
        }
        Err(e) => {
            println!("{} {}", "Failed to serialize backup:".red(), e);
        }
    }
}

fn show_device_info(state: &mut AppState) {
    if !check_device_connected(state) {
        println!("{}", "Error: No device connected!".red());
        return;
    }

    println!();
    println!("{}", "=========================================".cyan());
    println!("{}", "  Device Information".cyan().bold());
    println!("{}", "=========================================".cyan());

    let info_commands = vec![
        ("Device Model", &["shell", "getprop", "ro.product.model"][..]),
        ("Manufacturer", &["shell", "getprop", "ro.product.manufacturer"]),
        ("Android Version", &["shell", "getprop", "ro.build.version.release"]),
        ("SDK Version", &["shell", "getprop", "ro.build.version.sdk"]),
        ("Serial Number", &["shell", "getprop", "ro.serialno"]),
    ];

    for (label, args) in info_commands {
        match execute_adb_command(args) {
            Ok(output) => {
                let value = output.trim();
                if !value.is_empty() {
                    println!("{}: {}", label.cyan(), value.bright_white());
                }
            }
            Err(_) => {}
        }
    }

    println!("{}", "=========================================".cyan());
}