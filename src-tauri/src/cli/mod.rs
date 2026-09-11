use crate::models::{CliExecuteRequest, CliExecuteResponse};
use std::process::Stdio;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub async fn run_cli_command(req: CliExecuteRequest) -> Result<CliExecuteResponse, String> {
    let start_time = Instant::now();
    let timeout_duration = Duration::from_millis(req.timeout_ms.unwrap_or(10_000));

    let mut cmd = Command::new(&req.command);
    cmd.args(&req.args);

    if let Some(ref dir) = req.working_dir {
        if !dir.is_empty() {
            cmd.current_dir(dir);
        }
    }

    if let Some(ref envs) = req.env_vars {
        for (k, v) in envs {
            cmd.env(k, v);
        }
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    if req.stdin_content.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn command '{}': {}", req.command, e))?;

    // 如果有 stdin 内容，异步写入
    if let Some(stdin_data) = req.stdin_content {
        if let Some(mut stdin) = child.stdin.take() {
            tokio::spawn(async move {
                let _ = stdin.write_all(stdin_data.as_bytes()).await;
                let _ = stdin.flush().await;
            });
        }
    }

    // 等待命令执行，带超时控制
    let wait_res = timeout(timeout_duration, child.wait_with_output()).await;

    let output = match wait_res {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => return Err(format!("Command execution error: {}", e)),
        Err(_) => return Err(format!("Command timed out after {} ms", timeout_duration.as_millis())),
    };

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(CliExecuteResponse {
        exit_code: output.status.code(),
        stdout,
        stderr,
        duration_ms,
    })
}

