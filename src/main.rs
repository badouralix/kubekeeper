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

    false
}

/// Return true iff context matches at least one context pattern in allowlist.
fn check_context(context: &str, allowlist: Vec<&str>) -> bool {
    for allowlist_context_pattern in allowlist {
        if check_context_against_pattern(context, allowlist_context_pattern) {
            return true;
        }
    }

    false
}

/// Return true iff context matches pattern.
///
/// A context contains regular chars. A pattern contains zero, one or many
/// wildcards, and regular chars. Wildcards can match zero, one or many regular
/// chars. For instance `kube-production-1` matches `*prod*`.
///
/// This algorithm is a tweaked version of the regex backtracking. In the worst
/// case scenario, it runs in `O(context.len() * pattern.len())`. It is inspired
/// by https://research.swtch.com/glob.
fn check_context_against_pattern(context: &str, pattern: &str) -> bool {
    // Store the context/pattern index of the current iteration
    let mut current_c_idx = 0;
    let mut current_p_idx = 0;
    // Store the context/pattern index to jump to when backtracking
    let mut backtrack_c_idx = 0;
    let mut backtrack_p_idx = 0;

    while current_c_idx < context.len() && backtrack_p_idx <= pattern.len() {
        if current_p_idx < pattern.len() {
            match pattern.as_bytes()[current_p_idx] {
                b'*' => {
                    backtrack_c_idx = current_c_idx + 1;
                    backtrack_p_idx = current_p_idx + 1;

                    current_p_idx += 1;

                    continue;
                }
                _ => {
                    if context.as_bytes()[current_c_idx] == pattern.as_bytes()[current_p_idx] {
                        current_c_idx += 1;
                        current_p_idx += 1;

                        continue;
                    }
                }
            }
        }

        // At this point, either the end of pattern was reached, or context does not match pattern
        // If a wildcard was encountered previously, then try to backtrack to the previous known wildcard
        if backtrack_p_idx != 0 {
            current_c_idx = backtrack_c_idx;
            current_p_idx = backtrack_p_idx;

            backtrack_c_idx += 1;

            continue;
        }

        return false;
    }

    // If context is shorter the pattern, we still want to return a match when pattern contains trailing wildcards
    if current_p_idx < pattern.len() {
        return pattern.as_bytes()[current_p_idx..]
            .iter()
            .all(|&char| char == b'*');
    }

    // No need to check the indices against the lengths since they are only incremented by one per iteration
    true
}

/// Return true iff context has already been validated earlier.
fn check_last_validation(context: &str) -> bool {
    let check_interval: u64 = env::var("KUBEKEEPER_CHECK_INTERVAL")
        .unwrap_or_else(|_| "900".to_string())
        .parse()
        .unwrap_or(900);
    let pidfile = env::temp_dir()
        .join(env::var("KUBEKEEPER_PIDFILE").unwrap_or_else(|_| "kubekeeper.pid".to_string()));

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
    if fs::read_to_string(pidfile).unwrap_or_default() != context {
        outdated = true;
    }

    !outdated
}

/// Return default include and exclude config.
#[allow(clippy::type_complexity)]
fn get_config() -> (
    HashMap<&'static str, Vec<&'static str>>,
    HashMap<&'static str, Vec<&'static str>>,
) {
    // These contexts and/or commands may _never_ require validation
    let mut exclude = HashMap::new();
    exclude.insert("context", vec!["kind-*", "minikube"]);
    exclude.insert(
        "command",
        vec![
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
    include.insert("context", vec!["*fed*", "*prod*"]);
    include.insert("command", vec!["apply", "delete", "edit", "label", "scale"]);

    (include, exclude)
}

/// Identify which actions must be taken: validation? record? amendment?
/// Return one boolean per question.
fn identify_actions(
    context: &str,
    command: &str,
    include: HashMap<&str, Vec<&str>>,
    exclude: HashMap<&str, Vec<&str>>,
) -> (bool, bool, bool) {
    // If command is empty or cobra dynamic completion, skip all actions
    // See https://github.com/spf13/cobra/blob/b9460cc/completions.go#L12-L19
    if command.is_empty() || command.starts_with("__complete") {
        return (false, false, false);
    }

    // If the context is set as an argument, skip all actions
    for arg in env::args() {
        if arg.starts_with("--context") {
            return (false, false, false);
        }
    }

    // Here we try to figure out if the command is a native kubectl command or a plugin
    // The --context option can only be prefixed to native commands
    // See https://github.com/kubernetes/kubernetes/pull/92343
    let amendment = if command.starts_with('-') {
        // If a global option is already provided before the command, then we assume it is a native command
        // It would be better to iterate over all args and exclude options with their value if any
        // But it would require to be able to handle cases where the option and the value are separated by spaces
        true
    } else {
        let native_kubectl_commands = String::from_utf8(
            Command::new("kubectl")
                .args(vec!["__completeNoDesc", ""])
                .output()
                .expect("failed to execute process")
                .stdout,
        )
        .unwrap();
        // It contains an extra ":4\n", but that merely affects the heuristic
        native_kubectl_commands.contains(&env::args().nth(1).unwrap())
    };

    if check_context(context, include["context"].clone()) {
        if check_command(command, exclude["command"].clone()) {
            // println!("Command '{command}' is excluded, skipping validation.");
            return (false, false, amendment);
        } else {
            // println!("Command '{command}' is not excluded, triggering validation.");
            return (true, true, amendment);
        }
    }

    if check_command(command, include["command"].clone()) {
        if check_context(context, exclude["context"].clone()) {
            return (false, false, amendment);
        } else {
            return (true, true, amendment);
        }
    }

    if check_context(context, exclude["context"].clone())
        || check_command(command, exclude["command"].clone())
    {
        return (false, false, amendment);
    }

    if check_last_validation(context) {
        return (false, true, amendment);
    }

    (true, true, amendment)
}

fn save_context(context: &str) -> std::io::Result<()> {
    let pidfile = env::temp_dir()
        .join(env::var("KUBEKEEPER_PIDFILE").unwrap_or_else(|_| "kubekeeper.pid".to_string()));

    fs::write(pidfile, context)
}

/// Instead of forking to kubectx, explicitly ask if the current context is correct.
fn validate_context(context: &str) -> std::io::Result<bool> {
    print!("Really run command in \x1b[1;93m{context}\x1b[0m? Press \"y\" to continue. Anything else will exit. ");
    std::io::stdout().flush()?;

    if let Ok(status) = Command::new("sh")
        .arg("-c")
        .arg("read -n1 && ([[ $REPLY != '' ]] && echo) && [[ $REPLY == 'y' ]]")
        .status()
    {
        return Ok(status.success());
    }

    // If executing a child process fails for some reasons, fallback to reading stdin
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    buffer = buffer.trim().to_string();
    Ok(buffer == "y")
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
    // println!("Decided validation={validation} record={record} amendment={amendment}");

    // Set new context if needed
    if validation {
        match validate_context(context.trim()) {
            Ok(true) => {}
            _ => {
                println!("Failed to validate context. Abort.");
                std::process::exit(1);
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
            .arg(format!("--context={}", context.trim()))
            .args(env::args().skip(1))
            .exec();
    } else {
        Command::new("kubectl").args(env::args().skip(1)).exec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_wildcard() {
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-production-1"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-production-2"),
            false,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-production-12"),
            false,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-prod"),
            false,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "production-1"),
            false,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-staging-1"),
            false,
        );
        assert_eq!(check_context_against_pattern("", ""), true);
    }

    #[test]
    fn single_wildcard() {
        assert_eq!(
            check_context_against_pattern("kube-production-1", "*kube-production-1"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-product*ion-1"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-production-1*"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "*-production-1"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-prod*"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "*-staging-1"),
            false,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-staging*"),
            false,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-*-1"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-*-2"),
            false,
        );
        assert_eq!(check_context_against_pattern("", "*"), true);
    }

    #[test]
    fn multiple_wildcards() {
        assert_eq!(
            check_context_against_pattern("kube-production-1", "kube-prod*-*"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "*prod*"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "**prod**"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "*"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "*****************"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "******************"),
            true,
        );
        assert_eq!(
            check_context_against_pattern("kube-production-1", "*staging*"),
            false,
        );
        assert_eq!(check_context_against_pattern("", "*prod*"), false);
    }
}
