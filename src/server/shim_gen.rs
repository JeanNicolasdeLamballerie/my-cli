use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::{config, tcp_log};

pub fn shims_exist() -> io::Result<bool> {
    let has_ps1 = crate::config::shims_ps_path()?.exists();
    let has_sh = crate::config::shims_sh_path()?.exists();
    Ok(has_ps1 && has_sh)
}

/// Render the shims into config_dir/dekharen-cli-daemon/shims.{psm1,sh}
///
/// `exe_path`: Path to the actual executable (e.g., from std::env::current_exe())
pub fn generate_shims(exe_path: &Path) -> io::Result<()> {
    let port_path = crate::config::port_file_path();
    // let config_dir = config_path().unwrap(); // TODO : Remove unwrap
    let exe_path_str = exe_path.to_string_lossy();

    fs::create_dir_all(port_path.parent().unwrap())?;

    // PowerShell Shim
    let psm1 = format!(
        r#"# PowerShell shim for dekharen-cli-daemon
function rush {{
    param([Parameter(ValueFromRemainingArguments = $true)][String[]]$args)
    $portPath = "{port_path}"
    if (-not (Test-Path $portPath)) {{
        Write-Host "Port file not found. Attempting to start daemon..."
        start-cli-daemon
        Start-Sleep -Seconds 1
    }}

    try {{
        $port = Get-Content $portPath
        $tcpClient = New-Object System.Net.Sockets.TcpClient("127.0.0.1", [int]$port)
        $stream = $tcpClient.GetStream()
        $writer = New-Object System.IO.StreamWriter $stream
        $writer.AutoFlush = $true
        $writer.WriteLine(($args -join ' '))
        $reader = New-Object System.IO.StreamReader $stream
        while (($line = $reader.ReadLine()) -ne $null) {{ Write-Output $line }}
    }} catch {{
        Write-Error "Failed to connect to the daemon. Is it running?"
    }}
}}

function dash {{
    param([string]$name)
    $result = rush move $name
    if (-not $result) {{
        Write-Error "dash failed. Could the daemon be down?"
        return
    }}
    Set-Location $result
}}

function start-cli-daemon {{
    param([int]$port = 5339)
    Start-Process -FilePath "{exe_path}" -ArgumentList "serve $port" -NoNewWindow
}}

Export-ModuleMember -Function rush, dash, start-cli-daemon
"#,
        port_path = port_path.display(),
        exe_path = exe_path_str,
    );

    fs::write(config::shims_ps_path()?, psm1)?;

    // Shell shim
    let sh = format!(
        r#"#!/bin/sh
# Shell shim for dekharen-cli-daemon
PORT_FILE="{port_path}"

rush() {{
    if [ ! -f "$PORT_FILE" ]; then
        echo "Port file not found. Attempting to start daemon..."
        start-cli-daemon
        sleep 1
    fi

    PORT=$(cat "$PORT_FILE")
    if ! exec 3<>/dev/tcp/127.0.0.1/$PORT; then
        echo "rush: failed to connect to daemon. Is it running?"
        return 1
    fi
    printf "%s\n" "$*" >&3
    cat <&3
}}

dash() {{
    target=$(rush move "$@")
    if [ -z "$target" ]; then
        echo "dash: failed to get path. Could the daemon be down?"
        return 1
    fi
    cd "$target"
}}

start-cli-daemon() {{
    PORT_ARG="$1"
    if [ -z "$PORT_ARG" ]; then
        PORT_ARG=5339
    fi
    "{exe_path}" serve "$PORT_ARG" &
}}
"#,
        port_path = port_path.display(),
        exe_path = exe_path_str,
    );

    // let sh_path = config_dir.join("shims.sh");
    let sh_path = crate::config::shims_sh_path()?;
    let mut file = fs::File::create(&sh_path)?;
    file.write_all(sh.as_bytes())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&sh_path, perms)?;
    }

    Ok(())
}

pub fn generate_completions(cli_command: &mut clap::Command) -> io::Result<()> {
    use clap_complete::{
        generate_to,
        shells::{Bash, PowerShell, Zsh},
    };

    let outdir = dirs::config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No config dir found"))?
        .join("dekharen-cli-daemon/completions");
    fs::create_dir_all(&outdir)?;

    generate_to(Bash, cli_command, "rush", &outdir)?;
    generate_to(Zsh, cli_command, "rush", &outdir)?;
    generate_to(PowerShell, cli_command, "rush", &outdir)?;

    Ok(())
}

pub fn print_completion_instructions() {
    if let Some(config_dir) = dirs::config_dir() {
        let base = config_dir.join("dekharen-cli-daemon/completions");
        tcp_log!("\n🔧 To enable tab-completion, add this to your configuration :\n");

        tcp_log!("  Bash:\n    source {}", base.join("rush.bash").display());
        tcp_log!(
            "\n  Zsh:\n    fpath=({} $fpath)\n    autoload -Uz compinit && compinit",
            base.display()
        );
        tcp_log!(
            "\n  PowerShell:\n    Import-Module {}",
            base.join("_rush.ps1").display()
        );
    }
}
