/// Basically tries to call `which ${binary_name}` & reports the command status.
/// Returns false if the check has failed for some other reason!
pub(crate) fn check_binary_exists(binary_name: &str) -> bool {
    let mut which_command = std::process::Command::new("which");
    which_command.arg(binary_name);

    let exit_status = match which_command.status() {
        Ok(status) => status,
        Err(err) => {
            println!(
                "Failed to determine whether the binary: {binary_name} exists with error: {err}"
            );
            return false;
        }
    };

    if exit_status.success() {
        return true;
    } else {
        return false;
    }
}
