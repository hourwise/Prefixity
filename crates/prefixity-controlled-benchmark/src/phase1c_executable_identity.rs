//! Stable, read-only identity for an executable file.
//!
//! Path spelling is retained for diagnostics, but it is not the security
//! identity.  The binding uses the resolved final path, file size, content
//! SHA-256, and (on Windows) the file identity returned from a file handle.

use crate::hashing::sha256_hex;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutableIdentity {
    /// The path representation supplied by the caller or native inspector.
    pub raw_path: String,
    /// The final path resolved from the executable file.
    pub final_path: String,
    pub file_size: u64,
    pub sha256: String,
    /// Windows volume serial plus file index.  This is absent on POSIX.
    pub file_id: Option<String>,
}

pub fn inspect(path: &Path) -> Result<ExecutableIdentity, String> {
    let raw_path = path.to_string_lossy().into_owned();
    let canonical_path = std::fs::canonicalize(path)
        .map_err(|error| format!("canonicalize executable path {raw_path:?}: {error}"))?;
    let metadata = std::fs::metadata(&canonical_path)
        .map_err(|error| format!("stat executable path {raw_path:?}: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("executable path {raw_path:?} is not a file"));
    }
    let bytes = std::fs::read(&canonical_path)
        .map_err(|error| format!("read executable path {raw_path:?}: {error}"))?;
    let (final_path, file_id) = platform_file_identity(path, &canonical_path)?;
    Ok(ExecutableIdentity {
        raw_path,
        final_path,
        file_size: metadata.len(),
        sha256: sha256_hex(&bytes),
        file_id,
    })
}

/// Compare executable objects without comparing the launch-path spelling.
pub fn same(actual: &ExecutableIdentity, expected: &ExecutableIdentity) -> bool {
    if actual.file_size != expected.file_size || actual.sha256 != expected.sha256 {
        return false;
    }
    match (&actual.file_id, &expected.file_id) {
        (Some(actual_id), Some(expected_id)) => actual_id == expected_id,
        (None, None) => final_paths_equal(&actual.final_path, &expected.final_path),
        _ => false,
    }
}

#[cfg(windows)]
fn final_paths_equal(actual: &str, expected: &str) -> bool {
    actual.eq_ignore_ascii_case(expected)
}

#[cfg(not(windows))]
fn final_paths_equal(actual: &str, expected: &str) -> bool {
    actual == expected
}

#[cfg(not(windows))]
fn platform_file_identity(
    _opened_path: &Path,
    canonical_path: &Path,
) -> Result<(String, Option<String>), String> {
    Ok((canonical_path.to_string_lossy().into_owned(), None))
}

#[cfg(windows)]
fn platform_file_identity(
    opened_path: &Path,
    _canonical_path: &Path,
) -> Result<(String, Option<String>), String> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, GetFileInformationByHandle, GetFinalPathNameByHandleW,
        BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_NORMAL, FILE_NAME_NORMALIZED,
        FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };

    let wide_path = opened_path
        .as_os_str()
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<_>>();
    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            null(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(format!(
            "CreateFileW failed for executable identity with Windows error {}",
            unsafe { GetLastError() }
        ));
    }

    let result = (|| {
        let mut path_buffer = [0u16; 32_768];
        let path_length = unsafe {
            GetFinalPathNameByHandleW(
                handle,
                path_buffer.as_mut_ptr(),
                path_buffer.len() as u32,
                FILE_NAME_NORMALIZED,
            )
        };
        if path_length == 0 || path_length as usize >= path_buffer.len() {
            return Err(format!(
                "GetFinalPathNameByHandleW failed with Windows error {}",
                unsafe { GetLastError() }
            ));
        }

        let mut file_information = BY_HANDLE_FILE_INFORMATION::default();
        if unsafe { GetFileInformationByHandle(handle, &mut file_information) } == 0 {
            return Err(format!(
                "GetFileInformationByHandle failed with Windows error {}",
                unsafe { GetLastError() }
            ));
        }
        let final_path = String::from_utf16_lossy(&path_buffer[..path_length as usize]);
        let file_id = format!(
            "volume={:08x};index={:08x}{:08x}",
            file_information.dwVolumeSerialNumber,
            file_information.nFileIndexHigh,
            file_information.nFileIndexLow
        );
        Ok((final_path, Some(file_id)))
    })();
    unsafe { CloseHandle(handle) };
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn alternate_path_representation_accepts_same_file() {
        let root = std::env::temp_dir().join(format!(
            "prefixity-workflow-identity-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let first = root.join("child-a.exe");
        let second = root.join("child-alias.exe");
        fs::write(&first, b"same executable bytes").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&first, &second).unwrap();
        #[cfg(windows)]
        fs::hard_link(&first, &second).unwrap();

        let first_identity = inspect(&first).unwrap();
        let second_identity = inspect(&second).unwrap();
        assert!(same(&first_identity, &second_identity));
        assert_ne!(first_identity.raw_path, second_identity.raw_path);

        let _ = fs::remove_file(&second);
        let _ = fs::remove_file(&first);
        let _ = fs::remove_dir(&root);
    }

    #[test]
    fn different_file_with_same_basename_is_rejected() {
        let root = std::env::temp_dir().join(format!(
            "prefixity-workflow-identity-negative-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let first = root.join("child.exe");
        let second_root = root.join("other");
        fs::create_dir_all(&second_root).unwrap();
        let second = second_root.join("child.exe");
        fs::write(&first, b"first executable bytes").unwrap();
        fs::write(&second, b"different executable bytes").unwrap();

        let first_identity = inspect(&first).unwrap();
        let second_identity = inspect(&second).unwrap();
        assert!(!same(&first_identity, &second_identity));

        let _ = fs::remove_file(&second);
        let _ = fs::remove_dir(&second_root);
        let _ = fs::remove_file(&first);
        let _ = fs::remove_dir(&root);
    }
}
