use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::time::SystemTime;

/// Return true iff command uses a subcommand in allowlist.
fn check_command(command: &str, allowlist: Vec<&str>) -> bool {
    for allowlist_command_prefix in allowlist {
        if command.starts_with(allowlist_command_prefix) {
            return true;
        }
    }

    return false;
}

/// Return true iff context is in allowlist.
fn check_context(context: &str, allowlist: Vec<&str>) -> bool {
    return allowlist.contains(&context.trim());
}

/// Return true iff context has already been validated earlier.
fn check_last_validation(context: &str) -> bool {
    let check_interval: u64 = env::var("KUBEKEEPER_CHECK_INTERVAL")
        .unwrap_or("900".to_string())
        .parse()
        .unwrap_or(900);
    let pidfile = env::temp_dir()
        .join(env::var("KUBEKEEPER_PIDFILE").unwrap_or("kubekeeper.pid".to_string()));

    let mut outdated = false;

    match fs::metadata(pidfile.clone()) {
        Ok(metadata) => {
            if SystemTime::now()
                .duration_since(metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH))
                .unwrap()
                .as_secs()
                > check_interval
            {
                outdated = true;
            }
        }
        // We are conservative here, we assume we did not validate the context recently
        Err(_) => outdated = true,
    }

    // Use unwrap_or_default to ask for validation if reading from pidfile failed
    if fs::read_to_string(pidfile.clone()).unwrap_or_default() != context {
        outdated = true;
    }

    return !outdated;
}

/// Return default include and exclude config.
fn get_config() -> (
    HashMap<&'static str, Vec<&'static str>>,
    HashMap<&'static str, Vec<&'static str>>,
) {
    // These contexts and/or commands may _never_ require validation
    let mut exclude = HashMap::new();
    exclude.insert("context", vec!["minikube"]);
    exclude.insert(
        "command",
        vec![
            "__complete",
            "api-resources",
            "api-versions",
            "cluster-info",
            "completion",
            "config current-context",
            "config get-clusters",
            "config get-contexts",
            "config view",
            "describe",
            "diff",
            "explain",
            "get",
            "help",
            "logs",
            "options",
            "top",
            "version",
        ],
    );

    // These contexts and/or commands may _always_ require validation
    let mut include = HashMap::new();
    include.insert("context", vec![]);
    include.insert("command", vec!["apply", "delete", "scale"]);

    return (include, exclude);
}

/// Identify which actions must be taken: validation? record? amendment?
/// Return one boolean per question.
fn identify_actions(
    context: &str,
    command: &str,
    include: HashMap<&str, Vec<&str>>,
    exclude: HashMap<&str, Vec<&str>>,
) -> (bool, bool, bool) {
    // If command is empty, skip all actions
    if command == "" {
        return (false, false, false);
    }

    // If the context is set as an argument, skip all actions
    for arg in env::args() {
        if arg.starts_with("--context") {
            return (false, false, false);
        }
    }

    if check_context(context, include["context"].clone()) {
        if check_command(command, exclude["command"].clone()) {
            // println!("Command '{command}' is excluded, skipping validation.");
            return (false, false, true);
        } else {
            // println!("Command '{command}' is not excluded, triggering validation.");
            return (true, true, true);
        }
    }

    if check_command(command, include["command"].clone()) {
        if check_context(context, exclude["context"].clone()) {
            return (false, false, true);
        } else {
            return (true, true, true);
        }
    }

    if check_context(context, exclude["context"].clone())
        || check_command(command, exclude["command"].clone())
    {
        return (false, false, true);
    }

    if check_last_validation(context) {
        return (false, true, true);
    }

    return (true, true, true);
}

fn save_context(context: &str) -> std::io::Result<()> {
    let pidfile = env::temp_dir()
        .join(env::var("KUBEKEEPER_PIDFILE").unwrap_or("kubekeeper.pid".to_string()));

    fs::write(pidfile, context)?;
    Ok(())
}

/// Instead of forking to kubectx, explicitly ask if the current context is correct.
fn validate_context(context: &str) -> std::io::Result<bool> {
    print!("Really run command in \x1b[93m{context}\x1b[0m? Enter \"yes\" to continue. Anything else will exit. ");
    std::io::stdout().flush()?;

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    buffer = buffer.trim().to_string();

    if buffer != "yes" {
        return Ok(false);
    }

    Ok(true)
}

fn main() {
    // Parse configuration
    let (include, exclude) = get_config();

    // Figure out what to do
    let context = String::from_utf8(
        Command::new("kubectl")
            .arg("config")
            .arg("current-context")
            .output()
            .expect("failed to execute process")
            .stdout,
    )
    .unwrap();
    // println!("Found context={}", context.trim());
    let command = env::args().skip(1).collect::<Vec<String>>().join(" ");
    // println!("Received command={command}");
    let (validation, record, amendment) = identify_actions(&context, &command, include, exclude);
    // println!("Decided validation={validation} record={record}");

    // Set new context if needed
    if validation {
        match validate_context(context.trim()) {
            Ok(true) => {}
            _ => {
                println!("Failed to validate context. Abort.");
                return;
            }
        }
    }

    // Save new context to prevent revalidation
    if record {
        // Use unwrap_or_default to do nothing if writing to pidfile failed
        save_context(&context).unwrap_or_default();
    }

    // Run kubectl with context
    if amendment {
        Command::new("kubectl")
            .args(env::args().skip(1))
            .args(vec!["--context", context.trim()])
            .exec();
    } else {
        Command::new("kubectl").args(env::args().skip(1)).exec();
    }
}
