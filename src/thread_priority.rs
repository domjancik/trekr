#[cfg(target_os = "windows")]
pub(crate) fn promote_current_thread_for_midi(role: &str) -> Result<(), String> {
    use std::ffi::c_void;

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn GetCurrentThread() -> *mut c_void;
        fn SetThreadPriority(h_thread: *mut c_void, n_priority: i32) -> i32;
    }

    const THREAD_PRIORITY_HIGHEST: i32 = 2;

    let ok = unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST) };
    if ok == 0 {
        Err(format!("failed to raise {role} thread priority"))
    } else {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn promote_current_thread_for_midi(role: &str) -> Result<(), String> {
    use libc::{PRIO_PROCESS, SCHED_RR, c_int, pthread_self, sched_param};

    let tid = unsafe { libc::syscall(libc::SYS_gettid) as libc::id_t };
    let mut errors = Vec::new();

    let nice_result = unsafe { libc::setpriority(PRIO_PROCESS, tid as u32, -10) };
    if nice_result != 0 {
        errors.push(format!(
            "setpriority errno={}",
            std::io::Error::last_os_error()
        ));
    }

    let params = sched_param { sched_priority: 1 };
    let sched_result =
        unsafe { libc::pthread_setschedparam(pthread_self(), SCHED_RR, &params as *const _) };
    if sched_result != 0 {
        errors.push(format!("pthread_setschedparam errno={sched_result}"));
    }

    if errors.len() == 2 {
        Err(format!(
            "failed to raise {role} thread priority: {}",
            errors.join(", ")
        ))
    } else {
        Ok(())
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub(crate) fn promote_current_thread_for_midi(_role: &str) -> Result<(), String> {
    Ok(())
}
