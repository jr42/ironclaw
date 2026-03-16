//! Boot screen displayed after all initialization completes.
//!
//! Shows a polished ANSI-styled status panel summarizing the agent's runtime
//! state: model, database, tool count, enabled features, active channels,
//! and the gateway URL.

use crate::cli::fmt;

/// All displayable fields for the boot screen.
pub struct BootInfo {
    pub version: String,
    pub agent_name: String,
    pub llm_backend: String,
    pub llm_model: String,
    pub cheap_model: Option<String>,
    pub db_backend: String,
    pub db_connected: bool,
    pub tool_count: usize,
    pub gateway_url: Option<String>,
    pub embeddings_enabled: bool,
    pub embeddings_provider: Option<String>,
    pub heartbeat_enabled: bool,
    pub heartbeat_interval_secs: u64,
    pub sandbox_enabled: bool,
    pub docker_status: crate::sandbox::detect::DockerStatus,
    pub claude_code_enabled: bool,
    pub routines_enabled: bool,
    pub skills_enabled: bool,
    pub channels: Vec<String>,
    /// Public URL from a managed tunnel (e.g., "https://abc.ngrok.io").
    pub tunnel_url: Option<String>,
    /// Provider name for the managed tunnel (e.g., "ngrok").
    pub tunnel_provider: Option<String>,
    /// Time elapsed during startup. Shown at the bottom when present.
    pub startup_elapsed: Option<std::time::Duration>,
}

const KW: usize = 10;

/// Print the boot screen to stdout.
pub fn print_boot_screen(info: &BootInfo) {
    let border = format!("  {}", fmt::separator(58));

    println!();
    println!("{border}");
    println!();
    println!(
        "  {}{}{} v{}",
        fmt::bold(),
        info.agent_name,
        fmt::reset(),
        info.version
    );
    println!();

    // Model line — complex (multiple styled parts), so manual formatting
    let model_display = if let Some(ref cheap) = info.cheap_model {
        format!(
            "{}{}{}  {}cheap{} {}{}{}",
            fmt::accent(),
            info.llm_model,
            fmt::reset(),
            fmt::dim(),
            fmt::reset(),
            fmt::accent(),
            cheap,
            fmt::reset(),
        )
    } else {
        format!("{}{}{}", fmt::accent(), info.llm_model, fmt::reset())
    };
    println!(
        "  {}{:<width$}{}  {model_display}  {}via {}{}",
        fmt::dim(),
        "model",
        fmt::reset(),
        fmt::dim(),
        info.llm_backend,
        fmt::reset(),
        width = KW,
    );

    // Database line
    let db_status = if info.db_connected {
        "connected"
    } else {
        "none"
    };
    let db_value = format!("{} ({})", info.db_backend, db_status);
    println!("{}", fmt::kv_line("database", &db_value, KW));

    // Tools line
    let tools_value = format!("{} registered", info.tool_count);
    println!("{}", fmt::kv_line("tools", &tools_value, KW));

    // Features line — complex (multiple items, some with warnings), so manual formatting
    let mut features = Vec::new();
    if info.embeddings_enabled {
        if let Some(ref provider) = info.embeddings_provider {
            features.push(format!("embeddings ({provider})"));
        } else {
            features.push("embeddings".to_string());
        }
    }
    if info.heartbeat_enabled {
        let mins = info.heartbeat_interval_secs / 60;
        features.push(format!("heartbeat ({mins}m)"));
    }
    match info.docker_status {
        crate::sandbox::detect::DockerStatus::Available => {
            features.push("sandbox".to_string());
        }
        crate::sandbox::detect::DockerStatus::NotInstalled => {
            features.push(format!(
                "{}sandbox (docker not installed){}",
                fmt::warning(),
                fmt::reset()
            ));
        }
        crate::sandbox::detect::DockerStatus::NotRunning => {
            features.push(format!(
                "{}sandbox (docker not running){}",
                fmt::warning(),
                fmt::reset()
            ));
        }
        crate::sandbox::detect::DockerStatus::Disabled => {
            // Don't show sandbox when disabled
        }
    }
    if info.claude_code_enabled {
        features.push("claude-code".to_string());
    }
    if info.routines_enabled {
        features.push("routines".to_string());
    }
    if info.skills_enabled {
        features.push("skills".to_string());
    }
    if !features.is_empty() {
        println!(
            "  {}{:<width$}{}  {}{}{}",
            fmt::dim(),
            "features",
            fmt::reset(),
            fmt::accent(),
            features.join("  "),
            fmt::reset(),
            width = KW,
        );
    }

    // Channels line
    if !info.channels.is_empty() {
        let channels_value = info.channels.join("  ");
        println!(
            "  {}{:<width$}{}  {}{}{}",
            fmt::dim(),
            "channels",
            fmt::reset(),
            fmt::accent(),
            channels_value,
            fmt::reset(),
            width = KW,
        );
    }

    // Gateway URL (highlighted)
    if let Some(ref url) = info.gateway_url {
        println!();
        println!(
            "  {}{:<width$}{}  {}{}{}",
            fmt::dim(),
            "gateway",
            fmt::reset(),
            fmt::link(),
            url,
            fmt::reset(),
            width = KW,
        );
    }

    // Tunnel URL
    if let Some(ref url) = info.tunnel_url {
        let provider_tag = info
            .tunnel_provider
            .as_deref()
            .map(|p| format!(" {}({}){}", fmt::dim(), p, fmt::reset()))
            .unwrap_or_default();
        println!(
            "  {}{:<width$}{}  {}{}{}{}",
            fmt::dim(),
            "tunnel",
            fmt::reset(),
            fmt::link(),
            url,
            fmt::reset(),
            provider_tag,
            width = KW,
        );
    }

    println!();
    println!("{border}");

    // Startup elapsed
    if let Some(elapsed) = info.startup_elapsed {
        let millis = elapsed.as_millis();
        let elapsed_str = if millis < 1000 {
            format!("{millis}ms")
        } else {
            let secs = elapsed.as_secs_f64();
            format!("{secs:.1}s")
        };
        println!("  {}ready in {}{}", fmt::dim(), elapsed_str, fmt::reset());
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::detect::DockerStatus;

    #[test]
    fn test_print_boot_screen_full() {
        let info = BootInfo {
            version: "0.2.0".to_string(),
            agent_name: "ironclaw".to_string(),
            llm_backend: "nearai".to_string(),
            llm_model: "claude-3-5-sonnet-20241022".to_string(),
            cheap_model: Some("gpt-4o-mini".to_string()),
            db_backend: "libsql".to_string(),
            db_connected: true,
            tool_count: 24,
            gateway_url: Some("http://127.0.0.1:3001/?token=abc123".to_string()),
            embeddings_enabled: true,
            embeddings_provider: Some("openai".to_string()),
            heartbeat_enabled: true,
            heartbeat_interval_secs: 1800,
            sandbox_enabled: true,
            docker_status: DockerStatus::Available,
            claude_code_enabled: false,
            routines_enabled: true,
            skills_enabled: true,
            channels: vec![
                "repl".to_string(),
                "gateway".to_string(),
                "telegram".to_string(),
            ],
            tunnel_url: Some("https://abc123.ngrok.io".to_string()),
            tunnel_provider: Some("ngrok".to_string()),
            startup_elapsed: None,
        };
        // Should not panic
        print_boot_screen(&info);
    }

    #[test]
    fn test_print_boot_screen_minimal() {
        let info = BootInfo {
            version: "0.2.0".to_string(),
            agent_name: "ironclaw".to_string(),
            llm_backend: "nearai".to_string(),
            llm_model: "gpt-4o".to_string(),
            cheap_model: None,
            db_backend: "none".to_string(),
            db_connected: false,
            tool_count: 5,
            gateway_url: None,
            embeddings_enabled: false,
            embeddings_provider: None,
            heartbeat_enabled: false,
            heartbeat_interval_secs: 0,
            sandbox_enabled: false,
            docker_status: DockerStatus::Disabled,
            claude_code_enabled: false,
            routines_enabled: false,
            skills_enabled: false,
            channels: vec![],
            tunnel_url: None,
            tunnel_provider: None,
            startup_elapsed: None,
        };
        // Should not panic
        print_boot_screen(&info);
    }

    #[test]
    fn test_print_boot_screen_no_features() {
        let info = BootInfo {
            version: "0.1.0".to_string(),
            agent_name: "test".to_string(),
            llm_backend: "openai".to_string(),
            llm_model: "gpt-4o".to_string(),
            cheap_model: None,
            db_backend: "postgres".to_string(),
            db_connected: true,
            tool_count: 10,
            gateway_url: None,
            embeddings_enabled: false,
            embeddings_provider: None,
            heartbeat_enabled: false,
            heartbeat_interval_secs: 0,
            sandbox_enabled: false,
            docker_status: DockerStatus::Disabled,
            claude_code_enabled: false,
            routines_enabled: false,
            skills_enabled: false,
            channels: vec!["repl".to_string()],
            tunnel_url: None,
            tunnel_provider: None,
            startup_elapsed: None,
        };
        // Should not panic
        print_boot_screen(&info);
    }
}
