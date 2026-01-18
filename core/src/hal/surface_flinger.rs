//! Author: [Seclususs](https://github.com/seclususs)

use crate::daemon::types::QosError;

use std::process::{Command, Stdio};

const CMD_SERVICE: &str = "/system/bin/service";
const SVC_SF: &str = "SurfaceFlinger";
const TX_CODE: &str = "1035";

pub fn set_refresh_rate(param: i32) -> Result<(), QosError> {
    let status = Command::new(CMD_SERVICE)
        .arg("call")
        .arg(SVC_SF)
        .arg(TX_CODE)
        .arg("i32")
        .arg(param.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| QosError::SystemCheckFailed(format!("Exec failed: {}", e)))?;
    if status.success() {
        Ok(())
    } else {
        Err(QosError::SystemCheckFailed(format!(
            "SF Transaction rejected. Exit code: {:?}",
            status.code()
        )))
    }
}