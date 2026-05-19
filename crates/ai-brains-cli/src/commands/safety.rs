use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ChangeGuardHotspot {
    path: String,
    score: f64,
    complexity: u32,
    frequency: u32,
}

pub fn run(
    ctx: &crate::context::AppContext,
    limit: usize,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Scanning for ChangeGuard Hotspots...");

    // Try structured JSON output first; fall back to text mode
    let hotspots = match fetch_hotspots_json(limit) {
        Ok(hs) => {
            println!("ChangeGuard scan complete: {} hotspots found.", hs.len());
            hs
        }
        Err(json_err) => {
            // Fall back to text-mode parsing for older ChangeGuard versions
            match fetch_hotspots_text(limit) {
                Ok(hs) => {
                    println!(
                        "ChangeGuard scan complete (text mode, --json not available: {}).",
                        json_err
                    );
                    hs
                }
                Err(text_err) => {
                    return Err(format!(
                        "ChangeGuard scan failed. Ensure ChangeGuard is installed and initialized.\n\
                         JSON error: {}\nText error: {}",
                        json_err, text_err
                    )
                    .into());
                }
            }
        }
    };

    if hotspots.is_empty() {
        println!("No hotspots identified. Safety layer is healthy.");
        return Ok(());
    }

    if dry_run {
        println!("--- Dry Run: would sync {} hotspot(s) ---", hotspots.len());
        for (i, h) in hotspots.iter().enumerate() {
            println!(
                "  {}. {} (score: {:.2}, freq: {}, complexity: {})",
                i + 1,
                h.path,
                h.score,
                h.frequency,
                h.complexity
            );
        }
        println!("--- End Dry Run ---");
        return Ok(());
    }

    println!("Syncing top {} hotspot(s) to vault...", hotspots.len());

    let content = render_hotspots(&hotspots);

    super::pin::run(
        ctx,
        content,
        "assistant".to_string(),
        "LocalOnly".to_string(),
        Vec::new(),
        None,
    )?;

    println!("Safety synchronization complete.");
    Ok(())
}

fn fetch_hotspots_json(limit: usize) -> Result<Vec<ChangeGuardHotspot>, String> {
    let output = std::process::Command::new("changeguard")
        .args(["hotspots", "--json", "--limit", &limit.to_string()])
        .output()
        .map_err(|e| format!("failed to run changeguard: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("changeguard exited with error: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Log lines (with ANSI escapes containing '[') precede the JSON array on stdout.
    // Find the line that begins with '[' — that is the start of the JSON array.
    let json_start = stdout
        .lines()
        .position(|line| line.trim_start().starts_with('['))
        .ok_or_else(|| "no JSON array found in changeguard output".to_string())?;
    let json_str: String = stdout
        .lines()
        .skip(json_start)
        .collect::<Vec<_>>()
        .join("\n");

    serde_json::from_str::<Vec<ChangeGuardHotspot>>(&json_str)
        .map_err(|e| format!("failed to parse changeguard JSON: {}", e))
}

fn fetch_hotspots_text(limit: usize) -> Result<Vec<ChangeGuardHotspot>, String> {
    let output = std::process::Command::new("changeguard")
        .args(["hotspots", "--limit", &limit.to_string()])
        .output()
        .map_err(|e| format!("failed to run changeguard: {}", e))?;

    if !output.status.success() {
        return Err("changeguard exited with non-zero status".to_string());
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    // Parse markdown table: extract rows with | rank | score | freq | comp | path |
    let mut hotspots = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || trimmed.contains("---") || trimmed.contains("Rank") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
        if parts.len() >= 6 {
            if let (Ok(score), Ok(frequency), Ok(complexity)) = (
                parts[2].parse::<f64>(),
                parts[3].parse::<u32>(),
                parts[4].parse::<u32>(),
            ) {
                let path = parts[5].to_string();
                if !path.is_empty() && path != "File Path" {
                    hotspots.push(ChangeGuardHotspot {
                        path,
                        score,
                        complexity,
                        frequency,
                    });
                }
            }
        }
    }

    if hotspots.is_empty() {
        return Err("no hotspot rows found in text output".to_string());
    }
    Ok(hotspots)
}

fn render_hotspots(hotspots: &[ChangeGuardHotspot]) -> String {
    let mut lines = vec!["HOTSPOT: Brittle files identified by ChangeGuard:".to_string()];
    for (i, h) in hotspots.iter().enumerate() {
        lines.push(format!(
            "{}. {} (score: {:.2}, freq: {}, complexity: {})",
            i + 1,
            h.path,
            h.score,
            h.frequency,
            h.complexity
        ));
    }
    lines.join("\n")
}
