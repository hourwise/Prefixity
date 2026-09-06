//! Windows-native, read-only runtime exclusivity inspection.
//!
//! This module deliberately has no process-control, elevation, or localhost
//! surface. The Windows implementation reads the process table through
//! Toolhelp32 and TCP listener ownership through GetExtendedTcpTable. The
//! classification functions are platform-independent so their fail-closed
//! behavior can be tested without starting a model server.

use serde::Serialize;
use std::fmt;

pub const LLAMA_PROCESS_IMAGE: &str = "llama.exe";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProcessRecord {
    pub image_name: String,
    pub pid: u32,
    pub parent_pid: u32,
    pub executable_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExpectedProcessIdentity {
    pub pid: u32,
    pub executable_path: String,
    pub parent_pid: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExpectedWorkflowIdentity {
    pub launch_identity: String,
    pub supervisor: ExpectedProcessIdentity,
    pub calibration_child: ExpectedProcessIdentity,
    pub inspector: Option<ExpectedProcessIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListenerRecord {
    pub local_address: String,
    pub local_port: u16,
    pub pid: u32,
    pub state: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeFailureKind {
    ProcessInspection,
    PortInspection,
    #[cfg(windows)]
    ExecutablePath,
}

impl ProbeFailureKind {
    pub const fn outcome(self) -> &'static str {
        match self {
            Self::ProcessInspection => "PROCESS_INSPECTION_FAILED",
            Self::PortInspection => "PORT_INSPECTION_FAILED",
            #[cfg(windows)]
            Self::ExecutablePath => "EXECUTABLE_PATH_FAILED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeFailure {
    pub kind: ProbeFailureKind,
    pub message: String,
}

impl ProbeFailure {
    pub fn new(kind: ProbeFailureKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for ProbeFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.kind.outcome(), self.message)
    }
}

pub type ProbeResult<T> = Result<T, ProbeFailure>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExclusivityOutcome {
    ExclusivePrestart,
    ExclusivePoststart,
    LlamaAlreadyRunning,
    MultipleLlamaProcesses,
    ExpectedLlamaNotFound,
    PortAlreadyOwned,
    PortNotOwned,
    MultipleOrInconsistentListeners,
    PidPortMismatch,
    CompetingWorkflowProcess,
    ExpectedWorkflowIdentityInvalid,
    ExpectedSupervisorPidMissing,
    ExpectedSupervisorPathMismatch,
    ExpectedChildPathMismatch,
    UnexpectedWorkflowProcess,
    OperatorAttestationMissing,
    ProcessInspectionFailed,
    PortInspectionFailed,
    #[cfg(windows)]
    ExecutablePathFailed,
}

impl ExclusivityOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExclusivePrestart => "EXCLUSIVE_PRESTART",
            Self::ExclusivePoststart => "EXCLUSIVE_POSTSTART",
            Self::LlamaAlreadyRunning => "LLAMA_ALREADY_RUNNING",
            Self::MultipleLlamaProcesses => "MULTIPLE_LLAMA_PROCESSES",
            Self::ExpectedLlamaNotFound => "EXPECTED_LLAMA_NOT_FOUND",
            Self::PortAlreadyOwned => "PORT_ALREADY_OWNED",
            Self::PortNotOwned => "PORT_NOT_OWNED",
            Self::MultipleOrInconsistentListeners => "MULTIPLE_OR_INCONSISTENT_LISTENERS",
            Self::PidPortMismatch => "PID_PORT_MISMATCH",
            Self::CompetingWorkflowProcess => "COMPETING_WORKFLOW_PROCESS",
            Self::ExpectedWorkflowIdentityInvalid => "EXPECTED_WORKFLOW_IDENTITY_INVALID",
            Self::ExpectedSupervisorPidMissing => "EXPECTED_SUPERVISOR_PID_MISSING",
            Self::ExpectedSupervisorPathMismatch => "EXPECTED_SUPERVISOR_PATH_MISMATCH",
            Self::ExpectedChildPathMismatch => "EXPECTED_CHILD_PATH_MISMATCH",
            Self::UnexpectedWorkflowProcess => "UNEXPECTED_WORKFLOW_PROCESS",
            Self::OperatorAttestationMissing => "OPERATOR_ATTESTATION_MISSING",
            Self::ProcessInspectionFailed => "PROCESS_INSPECTION_FAILED",
            Self::PortInspectionFailed => "PORT_INSPECTION_FAILED",
            #[cfg(windows)]
            Self::ExecutablePathFailed => "EXECUTABLE_PATH_FAILED",
        }
    }

    pub const fn is_ready(self) -> bool {
        matches!(self, Self::ExclusivePrestart | Self::ExclusivePoststart)
    }
}

pub fn llama_processes(processes: &[ProcessRecord]) -> Vec<ProcessRecord> {
    processes
        .iter()
        .filter(|process| is_llama_process(&process.image_name))
        .cloned()
        .collect()
}

pub fn competing_processes(processes: &[ProcessRecord], current_pid: u32) -> Vec<ProcessRecord> {
    processes
        .iter()
        .filter(|process| process.pid != current_pid && is_competing_process(&process.image_name))
        .cloned()
        .collect()
}

pub fn competing_processes_for_expected_workflow(
    processes: &[ProcessRecord],
    identity: &ExpectedWorkflowIdentity,
) -> Vec<ProcessRecord> {
    let expected_pids = expected_workflow_pids(identity);
    processes
        .iter()
        .filter(|process| {
            !expected_pids.contains(&process.pid) && is_competing_process(&process.image_name)
        })
        .cloned()
        .collect()
}

pub fn validate_expected_workflow_identity(
    processes: &[ProcessRecord],
    identity: &ExpectedWorkflowIdentity,
) -> Result<(), ExclusivityOutcome> {
    if identity.launch_identity.trim().is_empty()
        || identity.supervisor.pid == 0
        || identity.calibration_child.pid == 0
        || identity.supervisor.executable_path.trim().is_empty()
        || identity.calibration_child.executable_path.trim().is_empty()
    {
        return Err(ExclusivityOutcome::ExpectedWorkflowIdentityInvalid);
    }
    let expected_pids = expected_workflow_pids(identity);
    if expected_pids.len() != 2 + usize::from(identity.inspector.as_ref().is_some())
        || expected_pids
            .iter()
            .enumerate()
            .any(|(index, pid)| expected_pids[..index].contains(pid))
    {
        return Err(ExclusivityOutcome::ExpectedWorkflowIdentityInvalid);
    }

    let supervisor = processes
        .iter()
        .find(|process| process.pid == identity.supervisor.pid)
        .ok_or(ExclusivityOutcome::ExpectedSupervisorPidMissing)?;
    if !same_executable_path(
        supervisor.executable_path.as_deref(),
        &identity.supervisor.executable_path,
    ) {
        return Err(ExclusivityOutcome::ExpectedSupervisorPathMismatch);
    }
    if let Some(expected_parent_pid) = identity.supervisor.parent_pid {
        if supervisor.parent_pid != expected_parent_pid {
            return Err(ExclusivityOutcome::ExpectedWorkflowIdentityInvalid);
        }
    }

    let child = processes
        .iter()
        .find(|process| process.pid == identity.calibration_child.pid)
        .ok_or(ExclusivityOutcome::ExpectedWorkflowIdentityInvalid)?;
    if !same_executable_path(
        child.executable_path.as_deref(),
        &identity.calibration_child.executable_path,
    ) {
        return Err(ExclusivityOutcome::ExpectedChildPathMismatch);
    }
    if child.parent_pid != identity.supervisor.pid
        || identity
            .calibration_child
            .parent_pid
            .is_some_and(|parent_pid| parent_pid != identity.supervisor.pid)
    {
        return Err(ExclusivityOutcome::ExpectedWorkflowIdentityInvalid);
    }

    if let Some(inspector_identity) = &identity.inspector {
        let inspector = processes
            .iter()
            .find(|process| process.pid == inspector_identity.pid)
            .ok_or(ExclusivityOutcome::ExpectedWorkflowIdentityInvalid)?;
        if !same_executable_path(
            inspector.executable_path.as_deref(),
            &inspector_identity.executable_path,
        ) {
            return Err(ExclusivityOutcome::ExpectedWorkflowIdentityInvalid);
        }
        if let Some(expected_parent_pid) = inspector_identity.parent_pid {
            if inspector.parent_pid != expected_parent_pid {
                return Err(ExclusivityOutcome::ExpectedWorkflowIdentityInvalid);
            }
        }
    }

    Ok(())
}

pub fn classify_poststart_with_expected_workflow(
    expected_pid: u32,
    processes: &ProbeResult<Vec<ProcessRecord>>,
    listeners: &ProbeResult<Vec<ListenerRecord>>,
    expected_workflow: &ExpectedWorkflowIdentity,
) -> ExclusivityOutcome {
    let process_records = match processes {
        Ok(records) => records,
        Err(failure) => return failure_outcome(failure),
    };
    if let Err(failure) = listeners {
        return failure_outcome(failure);
    }
    if let Err(outcome) = validate_expected_workflow_identity(process_records, expected_workflow) {
        return outcome;
    }
    let outcome = classify_poststart(expected_pid, processes, listeners);
    if outcome != ExclusivityOutcome::ExclusivePoststart {
        return outcome;
    }
    if competing_processes_for_expected_workflow(process_records, expected_workflow).is_empty() {
        ExclusivityOutcome::ExclusivePoststart
    } else {
        ExclusivityOutcome::UnexpectedWorkflowProcess
    }
}

fn expected_workflow_pids(identity: &ExpectedWorkflowIdentity) -> Vec<u32> {
    let mut pids = vec![identity.supervisor.pid, identity.calibration_child.pid];
    if let Some(inspector) = &identity.inspector {
        pids.push(inspector.pid);
    }
    pids
}

fn same_executable_path(actual: Option<&str>, expected: &str) -> bool {
    actual.is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
}

pub fn is_llama_process(image_name: &str) -> bool {
    image_name.eq_ignore_ascii_case(LLAMA_PROCESS_IMAGE)
}

pub fn is_competing_process(image_name: &str) -> bool {
    let image_name = image_name.to_ascii_lowercase();
    ["prefixity", "qwen"]
        .iter()
        .any(|marker| image_name.contains(marker))
}

pub fn classify_prestart(
    processes: &ProbeResult<Vec<ProcessRecord>>,
    listeners: &ProbeResult<Vec<ListenerRecord>>,
    operator_attested: bool,
) -> ExclusivityOutcome {
    let process_records = match processes {
        Ok(records) => records,
        Err(failure) => return failure_outcome(failure),
    };
    if let Err(failure) = listeners {
        return failure_outcome(failure);
    }
    let listener_records = listeners.as_ref().expect("checked above");
    let llama_count = llama_processes(process_records).len();
    if llama_count > 1 {
        return ExclusivityOutcome::MultipleLlamaProcesses;
    }
    if llama_count == 1 {
        return ExclusivityOutcome::LlamaAlreadyRunning;
    }
    if !listener_records.is_empty() {
        return if listener_records.len() == 1 {
            ExclusivityOutcome::PortAlreadyOwned
        } else {
            ExclusivityOutcome::MultipleOrInconsistentListeners
        };
    }
    if !operator_attested {
        return ExclusivityOutcome::OperatorAttestationMissing;
    }
    ExclusivityOutcome::ExclusivePrestart
}

pub fn classify_poststart(
    expected_pid: u32,
    processes: &ProbeResult<Vec<ProcessRecord>>,
    listeners: &ProbeResult<Vec<ListenerRecord>>,
) -> ExclusivityOutcome {
    let process_records = match processes {
        Ok(records) => records,
        Err(failure) => return failure_outcome(failure),
    };
    if let Err(failure) = listeners {
        return failure_outcome(failure);
    }
    let listener_records = listeners.as_ref().expect("checked above");
    let llama_records = llama_processes(process_records);
    if llama_records.len() > 1 {
        return ExclusivityOutcome::MultipleLlamaProcesses;
    }
    let Some(llama) = llama_records.first() else {
        return ExclusivityOutcome::ExpectedLlamaNotFound;
    };
    if llama.pid != expected_pid {
        return ExclusivityOutcome::PidPortMismatch;
    }
    if listener_records.len() > 1 {
        return ExclusivityOutcome::MultipleOrInconsistentListeners;
    }
    let Some(listener) = listener_records.first() else {
        return ExclusivityOutcome::PortNotOwned;
    };
    if listener.pid != expected_pid {
        return ExclusivityOutcome::PidPortMismatch;
    }
    ExclusivityOutcome::ExclusivePoststart
}

fn failure_outcome(failure: &ProbeFailure) -> ExclusivityOutcome {
    match failure.kind {
        ProbeFailureKind::ProcessInspection => ExclusivityOutcome::ProcessInspectionFailed,
        ProbeFailureKind::PortInspection => ExclusivityOutcome::PortInspectionFailed,
        #[cfg(windows)]
        ProbeFailureKind::ExecutablePath => ExclusivityOutcome::ExecutablePathFailed,
    }
}

#[cfg(windows)]
mod native {
    use super::{ListenerRecord, ProbeFailure, ProbeFailureKind, ProbeResult, ProcessRecord};
    use std::mem::size_of;
    use std::net::Ipv4Addr;
    use std::ptr::{null_mut, read_unaligned};
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_FILES,
        INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCPROW_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
    };
    use windows_sys::Win32::Networking::WinSock::AF_INET;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    const MAX_PROCESS_RECORDS: usize = 16 * 1024;

    pub fn process_table() -> ProbeResult<Vec<ProcessRecord>> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(windows_failure(
                ProbeFailureKind::ProcessInspection,
                "CreateToolhelp32Snapshot",
            ));
        }

        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let first_ok = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
        if !first_ok {
            unsafe { CloseHandle(snapshot) };
            return Err(windows_failure(
                ProbeFailureKind::ProcessInspection,
                "Process32FirstW",
            ));
        }

        let mut records = Vec::new();
        loop {
            if records.len() >= MAX_PROCESS_RECORDS {
                unsafe { CloseHandle(snapshot) };
                return Err(ProbeFailure::new(
                    ProbeFailureKind::ProcessInspection,
                    "process table exceeded the bounded record limit",
                ));
            }
            let image_name = utf16z(&entry.szExeFile);
            let executable_path = if super::is_llama_process(&image_name)
                || super::is_competing_process(&image_name)
            {
                Some(query_executable_path(entry.th32ProcessID)?)
            } else {
                None
            };
            records.push(ProcessRecord {
                image_name,
                pid: entry.th32ProcessID,
                parent_pid: entry.th32ParentProcessID,
                executable_path,
            });

            if unsafe { Process32NextW(snapshot, &mut entry) } == 0 {
                let error = unsafe { GetLastError() };
                unsafe { CloseHandle(snapshot) };
                if error != ERROR_NO_MORE_FILES {
                    return Err(windows_failure_code(
                        ProbeFailureKind::ProcessInspection,
                        "Process32NextW",
                        error,
                    ));
                }
                break;
            }
        }
        Ok(records)
    }

    pub fn tcp_listener_table(port: u16) -> ProbeResult<Vec<ListenerRecord>> {
        let mut byte_count = 0u32;
        let first_status = unsafe {
            GetExtendedTcpTable(
                null_mut(),
                &mut byte_count,
                0,
                AF_INET as u32,
                TCP_TABLE_OWNER_PID_LISTENER,
                0,
            )
        };
        if first_status != ERROR_INSUFFICIENT_BUFFER {
            return Err(windows_failure_code(
                ProbeFailureKind::PortInspection,
                "GetExtendedTcpTable(size)",
                first_status,
            ));
        }
        let mut buffer = vec![0u8; byte_count as usize];
        let status = unsafe {
            GetExtendedTcpTable(
                buffer.as_mut_ptr().cast(),
                &mut byte_count,
                0,
                AF_INET as u32,
                TCP_TABLE_OWNER_PID_LISTENER,
                0,
            )
        };
        if status != 0 {
            return Err(windows_failure_code(
                ProbeFailureKind::PortInspection,
                "GetExtendedTcpTable(data)",
                status,
            ));
        }
        if byte_count as usize > buffer.len() || buffer.len() < size_of::<u32>() {
            return Err(ProbeFailure::new(
                ProbeFailureKind::PortInspection,
                "GetExtendedTcpTable returned an invalid table size",
            ));
        }

        let count = unsafe { read_unaligned(buffer.as_ptr().cast::<u32>()) as usize };
        let row_size = size_of::<MIB_TCPROW_OWNER_PID>();
        let header_size = size_of::<u32>();
        if count > (buffer.len() - header_size) / row_size {
            return Err(ProbeFailure::new(
                ProbeFailureKind::PortInspection,
                "GetExtendedTcpTable row count exceeded the returned buffer",
            ));
        }
        let first_row = unsafe { buffer.as_ptr().add(header_size) };
        let mut listeners = Vec::new();
        for index in 0..count {
            let row = unsafe {
                read_unaligned(
                    first_row
                        .add(index * row_size)
                        .cast::<MIB_TCPROW_OWNER_PID>(),
                )
            };
            let local_port = u16::from_be(row.dwLocalPort as u16);
            if local_port != port {
                continue;
            }
            listeners.push(ListenerRecord {
                local_address: Ipv4Addr::from(u32::from_be(row.dwLocalAddr)).to_string(),
                local_port,
                pid: row.dwOwningPid,
                state: "LISTENING".to_string(),
            });
        }
        Ok(listeners)
    }

    fn query_executable_path(pid: u32) -> ProbeResult<String> {
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if handle.is_null() {
            return Err(windows_failure(
                ProbeFailureKind::ExecutablePath,
                "OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)",
            ));
        }
        let mut buffer = [0u16; 32_768];
        let mut length = buffer.len() as u32;
        let success =
            unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length) } != 0;
        unsafe { CloseHandle(handle) };
        if !success || length == 0 {
            return Err(windows_failure(
                ProbeFailureKind::ExecutablePath,
                "QueryFullProcessImageNameW",
            ));
        }
        Ok(String::from_utf16_lossy(&buffer[..length as usize]))
    }

    fn utf16z(buffer: &[u16]) -> String {
        let length = buffer
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(buffer.len());
        String::from_utf16_lossy(&buffer[..length])
    }

    fn windows_failure(kind: ProbeFailureKind, operation: &str) -> ProbeFailure {
        windows_failure_code(kind, operation, unsafe { GetLastError() })
    }

    fn windows_failure_code(kind: ProbeFailureKind, operation: &str, code: u32) -> ProbeFailure {
        ProbeFailure::new(
            kind,
            format!("{operation} failed with Windows error {code}"),
        )
    }
}

#[cfg(not(windows))]
mod native {
    use super::{ListenerRecord, ProbeFailure, ProbeFailureKind, ProbeResult, ProcessRecord};

    pub fn process_table() -> ProbeResult<Vec<ProcessRecord>> {
        Err(ProbeFailure::new(
            ProbeFailureKind::ProcessInspection,
            "Windows-native process inspection is unavailable on this platform",
        ))
    }

    pub fn tcp_listener_table(_port: u16) -> ProbeResult<Vec<ListenerRecord>> {
        Err(ProbeFailure::new(
            ProbeFailureKind::PortInspection,
            "Windows-native TCP ownership inspection is unavailable on this platform",
        ))
    }
}

pub use native::{process_table, tcp_listener_table};

#[cfg(test)]
mod tests {
    use super::*;

    fn process(image_name: &str, pid: u32) -> ProcessRecord {
        ProcessRecord {
            image_name: image_name.to_string(),
            pid,
            parent_pid: 0,
            executable_path: None,
        }
    }

    fn listener(pid: u32) -> ListenerRecord {
        ListenerRecord {
            local_address: "127.0.0.1".to_string(),
            local_port: 8080,
            pid,
            state: "LISTENING".to_string(),
        }
    }

    fn process_with_path(
        image_name: &str,
        pid: u32,
        parent_pid: u32,
        executable_path: &str,
    ) -> ProcessRecord {
        ProcessRecord {
            image_name: image_name.to_string(),
            pid,
            parent_pid,
            executable_path: Some(executable_path.to_string()),
        }
    }

    fn expected_workflow() -> ExpectedWorkflowIdentity {
        ExpectedWorkflowIdentity {
            launch_identity: "attempt-003-launch-001".to_string(),
            supervisor: ExpectedProcessIdentity {
                pid: 10,
                executable_path: r"D:\Prefixity\prefixity-phase1c-live-supervisor.exe".to_string(),
                parent_pid: None,
            },
            calibration_child: ExpectedProcessIdentity {
                pid: 11,
                executable_path: r"D:\Prefixity\prefixity-phase1c-calibration.exe".to_string(),
                parent_pid: Some(10),
            },
            inspector: None,
        }
    }

    fn expected_workflow_processes() -> Vec<ProcessRecord> {
        vec![
            process_with_path(
                "prefixity-phase1c-live-supervisor.exe",
                10,
                1,
                r"D:\Prefixity\prefixity-phase1c-live-supervisor.exe",
            ),
            process_with_path(
                "prefixity-phase1c-calibration.exe",
                11,
                10,
                r"D:\Prefixity\prefixity-phase1c-calibration.exe",
            ),
            process_with_path("llama.exe", 20, 1, r"D:\llama\llama.exe"),
        ]
    }

    #[test]
    fn no_matching_process_or_listener_is_exclusive_prestart() {
        let processes = Ok(vec![process("explorer.exe", 10)]);
        let listeners = Ok(Vec::new());
        assert_eq!(
            classify_prestart(&processes, &listeners, true),
            ExclusivityOutcome::ExclusivePrestart
        );
    }

    #[test]
    fn multiple_llama_processes_fail_closed() {
        let processes = Ok(vec![process("llama.exe", 10), process("llama.exe", 11)]);
        let listeners = Ok(Vec::new());
        assert_eq!(
            classify_prestart(&processes, &listeners, true),
            ExclusivityOutcome::MultipleLlamaProcesses
        );
    }

    #[test]
    fn unrelated_port_owner_blocks_prestart() {
        let processes = Ok(vec![process("explorer.exe", 10)]);
        let listeners = Ok(vec![listener(99)]);
        assert_eq!(
            classify_prestart(&processes, &listeners, true),
            ExclusivityOutcome::PortAlreadyOwned
        );
    }

    #[test]
    fn poststart_pid_mismatch_fails_closed() {
        let processes = Ok(vec![process("llama.exe", 10)]);
        let listeners = Ok(vec![listener(11)]);
        assert_eq!(
            classify_poststart(10, &processes, &listeners),
            ExclusivityOutcome::PidPortMismatch
        );
    }

    #[test]
    fn inspection_failure_fails_closed() {
        let processes: ProbeResult<Vec<ProcessRecord>> = Err(ProbeFailure::new(
            ProbeFailureKind::ProcessInspection,
            "synthetic failure",
        ));
        let listeners = Ok(Vec::new());
        assert_eq!(
            classify_prestart(&processes, &listeners, true),
            ExclusivityOutcome::ProcessInspectionFailed
        );
        let processes = Ok(Vec::new());
        let listeners: ProbeResult<Vec<ListenerRecord>> = Err(ProbeFailure::new(
            ProbeFailureKind::PortInspection,
            "synthetic failure",
        ));
        assert_eq!(
            classify_prestart(&processes, &listeners, true),
            ExclusivityOutcome::PortInspectionFailed
        );
    }

    #[cfg(windows)]
    #[test]
    fn executable_path_failure_is_distinct() {
        let processes: ProbeResult<Vec<ProcessRecord>> = Err(ProbeFailure::new(
            ProbeFailureKind::ExecutablePath,
            "synthetic failure",
        ));
        let listeners = Ok(Vec::new());
        assert_eq!(
            classify_prestart(&processes, &listeners, true),
            ExclusivityOutcome::ExecutablePathFailed
        );
    }

    #[test]
    fn registered_attempt003_workflow_is_exclusive_poststart() {
        let processes = Ok(expected_workflow_processes());
        let listeners = Ok(vec![listener(20)]);
        assert_eq!(
            classify_poststart_with_expected_workflow(
                20,
                &processes,
                &listeners,
                &expected_workflow()
            ),
            ExclusivityOutcome::ExclusivePoststart
        );
    }

    #[test]
    fn second_supervisor_is_an_unexpected_workflow_process() {
        let mut records = expected_workflow_processes();
        records.push(process_with_path(
            "prefixity-phase1c-live-supervisor.exe",
            12,
            1,
            r"D:\Prefixity\prefixity-phase1c-live-supervisor.exe",
        ));
        let processes = Ok(records);
        let listeners = Ok(vec![listener(20)]);
        assert_eq!(
            classify_poststart_with_expected_workflow(
                20,
                &processes,
                &listeners,
                &expected_workflow()
            ),
            ExclusivityOutcome::UnexpectedWorkflowProcess
        );
    }

    #[test]
    fn unregistered_prefixity_runner_is_an_unexpected_workflow_process() {
        let mut records = expected_workflow_processes();
        records.push(process_with_path(
            "prefixity-unregistered-runner.exe",
            12,
            1,
            r"D:\Prefixity\prefixity-unregistered-runner.exe",
        ));
        let processes = Ok(records);
        let listeners = Ok(vec![listener(20)]);
        assert_eq!(
            classify_poststart_with_expected_workflow(
                20,
                &processes,
                &listeners,
                &expected_workflow()
            ),
            ExclusivityOutcome::UnexpectedWorkflowProcess
        );
    }

    #[test]
    fn expected_supervisor_path_mismatch_fails_closed() {
        let mut identity = expected_workflow();
        identity.supervisor.executable_path = r"D:\Other\supervisor.exe".to_string();
        let processes = Ok(expected_workflow_processes());
        let listeners = Ok(vec![listener(20)]);
        assert_eq!(
            classify_poststart_with_expected_workflow(20, &processes, &listeners, &identity),
            ExclusivityOutcome::ExpectedSupervisorPathMismatch
        );
    }

    #[test]
    fn expected_supervisor_absence_fails_closed() {
        let mut identity = expected_workflow();
        identity.supervisor.pid = 99;
        let processes = Ok(expected_workflow_processes());
        let listeners = Ok(vec![listener(20)]);
        assert_eq!(
            classify_poststart_with_expected_workflow(20, &processes, &listeners, &identity),
            ExclusivityOutcome::ExpectedSupervisorPidMissing
        );
    }

    #[test]
    fn expected_child_path_mismatch_fails_closed() {
        let mut identity = expected_workflow();
        identity.calibration_child.executable_path = r"D:\Other\child.exe".to_string();
        let processes = Ok(expected_workflow_processes());
        let listeners = Ok(vec![listener(20)]);
        assert_eq!(
            classify_poststart_with_expected_workflow(20, &processes, &listeners, &identity),
            ExclusivityOutcome::ExpectedChildPathMismatch
        );
    }
}
