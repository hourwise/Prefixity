//! Stable, read-only identity for an executable file.
//!
//! Path spelling is retained for diagnostics, but it is not the security
//! identity.  The binding uses the resolved final path, file size, content
//! SHA-256, and (on Windows) the file identity returned from a file handle.

use crate::hashing::sha256_hex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

/// The two executable objects that a registered workflow authorizes.
///
/// This is deliberately separate from process identity. A process record
/// binds a PID to an image at runtime; this binding records the objects that
/// were authorized before the workflow was launched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenExecutableBinding {
    pub supervisor: ExecutableIdentity,
    pub child: ExecutableIdentity,
}

impl FrozenExecutableBinding {
    /// Read the durable executable records used by preparation identities.
    ///
    /// Historical certification identities only recorded executable hashes.
    /// They intentionally return no binding here rather than silently
    /// falling back to path-only validation. New live identities must carry
    /// both complete executable objects.
    pub fn from_implementation_fingerprints(identity: &Value) -> Result<Option<Self>, String> {
        let fingerprints = identity
            .get("implementation_fingerprints")
            .and_then(Value::as_object);
        let Some(fingerprints) = fingerprints else {
            return Ok(None);
        };
        let supervisor = fingerprints.get("supervisor_binary");
        let child = fingerprints.get("child_binary");
        match (supervisor, child) {
            (None, None) => Ok(None),
            (Some(supervisor), Some(child)) => Ok(Some(Self {
                supervisor: Self::identity_from_json(supervisor, "supervisor_binary")?,
                child: Self::identity_from_json(child, "child_binary")?,
            })),
            _ => Err(
                "frozen executable identity must include both supervisor_binary and child_binary"
                    .to_string(),
            ),
        }
    }

    fn identity_from_json(value: &Value, label: &str) -> Result<ExecutableIdentity, String> {
        let object = value
            .as_object()
            .ok_or_else(|| format!("{label} frozen executable identity is not an object"))?;
        let string_field = |name: &str| {
            object
                .get(name)
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
                .ok_or_else(|| format!("{label} frozen executable identity is missing {name}"))
        };
        let file_size = object
            .get("file_size")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("{label} frozen executable identity is missing file_size"))?;
        let file_id = object
            .get("file_id")
            .or_else(|| object.get("windows_file_id"))
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string);
        Ok(ExecutableIdentity {
            raw_path: string_field("raw_path")?,
            final_path: string_field("final_path")?,
            file_size,
            sha256: string_field("sha256")?,
            file_id,
        })
    }
}

/// Compare the objects authorized during preparation with the objects that
/// are about to be used by the workflow.
pub fn validate_frozen_executable_binding(
    expected: &FrozenExecutableBinding,
    actual_supervisor: &ExecutableIdentity,
    actual_child: &ExecutableIdentity,
) -> Result<(), String> {
    let mut mismatches = Vec::new();
    if !same(actual_supervisor, &expected.supervisor) {
        mismatches.push("supervisor");
    }
    if !same(actual_child, &expected.child) {
        mismatches.push("child");
    }
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "FROZEN_EXECUTABLE_IDENTITY_MISMATCH: {} executable object differs from preparation",
            mismatches.join(" and ")
        ))
    }
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

/// Materialize an executable into a bounded frozen staging location.
///
/// The destination must not already exist. This prevents a later build or a
/// repeated preparation from silently replacing an object that an identity
/// already authorized. The returned identity is for the staged copy, not the
/// source object; on Windows its file ID is expected to differ from the
/// source's file ID.
pub fn freeze_copy(source: &Path, destination: &Path) -> Result<ExecutableIdentity, String> {
    if destination.exists() {
        return Err(format!(
            "frozen executable destination already exists: {}",
            destination.display()
        ));
    }
    let parent = destination.parent().ok_or_else(|| {
        format!(
            "frozen executable destination has no parent: {}",
            destination.display()
        )
    })?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("create frozen executable staging directory: {error}"))?;
    std::fs::copy(source, destination).map_err(|error| {
        format!(
            "copy executable {} to frozen staging {}: {error}",
            source.display(),
            destination.display()
        )
    })?;
    inspect(destination)
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

/// Serial number of the volume that contains `path`, resolved through the
/// volume mount point rather than a handle to the file. Tests use it as an
/// independent reference for the volume component of a Windows file ID.
#[cfg(all(windows, test))]
pub(crate) fn containing_volume_serial(path: &Path) -> Result<u32, String> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::Storage::FileSystem::{GetVolumeInformationW, GetVolumePathNameW};

    let wide_path = path
        .as_os_str()
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<_>>();
    let mut volume_root = [0u16; 32_768];
    if unsafe {
        GetVolumePathNameW(
            wide_path.as_ptr(),
            volume_root.as_mut_ptr(),
            volume_root.len() as u32,
        )
    } == 0
    {
        return Err(format!(
            "GetVolumePathNameW failed with Windows error {}",
            unsafe { GetLastError() }
        ));
    }
    let mut serial = 0u32;
    if unsafe {
        GetVolumeInformationW(
            volume_root.as_ptr(),
            null_mut(),
            0,
            &mut serial,
            null_mut(),
            null_mut(),
            null_mut(),
            0,
        )
    } == 0
    {
        return Err(format!(
            "GetVolumeInformationW failed with Windows error {}",
            unsafe { GetLastError() }
        ));
    }
    Ok(serial)
}

/// The volume component of a `volume=XXXXXXXX;index=YYYYYYYYYYYYYYYY` file ID.
#[cfg(test)]
pub(crate) fn file_id_volume(file_id: &str) -> Option<u32> {
    let (volume, index) = file_id.strip_prefix("volume=")?.split_once(";index=")?;
    let hex = |text: &str, width: usize| {
        text.len() == width
            && text
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    };
    if !hex(volume, 8) || !hex(index, 16) {
        return None;
    }
    u32::from_str_radix(volume, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Regression for the Attempt 009-011 llama.exe record, whose volume
    /// component was another volume's serial: the helper's volume must be the
    /// volume that contains the inspected file. The temporary directory and
    /// the test executable are on different volumes on the preparation host
    /// (C: and D:), so this also covers the cross-volume case there.
    #[cfg(windows)]
    #[test]
    fn windows_file_id_volume_is_the_containing_volume() {
        let root =
            std::env::temp_dir().join(format!("prefixity-volume-identity-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let temporary = root.join("object.bin");
        fs::write(&temporary, b"volume identity").unwrap();
        let executable = std::env::current_exe().unwrap();

        let mut serials = Vec::new();
        for path in [temporary.as_path(), executable.as_path()] {
            let identity = inspect(path).unwrap();
            let recorded = identity
                .file_id
                .as_deref()
                .and_then(file_id_volume)
                .unwrap_or_else(|| panic!("{} has no well-formed file ID", path.display()));
            let containing = containing_volume_serial(path).unwrap();
            assert_eq!(recorded, containing, "{}", path.display());
            serials.push(containing);
        }
        let _ = fs::remove_file(&temporary);
        let _ = fs::remove_dir(&root);
        eprintln!(
            "volume identity: temp={:08x} executable={:08x} cross_volume={}",
            serials[0],
            serials[1],
            serials[0] != serials[1]
        );
    }

    #[test]
    fn file_id_volume_parses_only_the_helper_format() {
        assert_eq!(
            file_id_volume("volume=c4c93b54;index=00060000001ea970"),
            Some(0xc4c9_3b54)
        );
        assert_eq!(file_id_volume("0x000000000000000000060000001ea970"), None);
        assert_eq!(
            file_id_volume("volume=C4C93B54;index=00060000001ea970"),
            None
        );
        assert_eq!(file_id_volume("volume=c4c93b54;index=1ea970"), None);
    }

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

    #[cfg(windows)]
    #[test]
    fn dos_and_extended_path_spellings_accept_same_windows_file_object() {
        let root =
            std::env::temp_dir().join(format!("prefixity-extended-path-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let file = root.join("child.exe");
        fs::write(&file, b"same executable bytes").unwrap();
        let extended = std::path::PathBuf::from(format!(r"\\?\{}", file.display()));
        let dos_identity = inspect(&file).unwrap();
        let extended_identity = inspect(&extended).unwrap();
        assert!(same(&dos_identity, &extended_identity));
        let _ = fs::remove_file(&file);
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

    #[test]
    fn exact_frozen_binding_accepts_equal_supervisor_and_child_objects() {
        let root =
            std::env::temp_dir().join(format!("prefixity-frozen-binding-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let supervisor = root.join("supervisor.exe");
        let child = root.join("child.exe");
        fs::write(&supervisor, b"supervisor bytes").unwrap();
        fs::write(&child, b"child bytes").unwrap();
        let expected = FrozenExecutableBinding {
            supervisor: inspect(&supervisor).unwrap(),
            child: inspect(&child).unwrap(),
        };
        let actual = FrozenExecutableBinding {
            supervisor: inspect(&supervisor).unwrap(),
            child: inspect(&child).unwrap(),
        };
        assert!(
            validate_frozen_executable_binding(&expected, &actual.supervisor, &actual.child)
                .is_ok()
        );
        let _ = fs::remove_file(&child);
        let _ = fs::remove_file(&supervisor);
        let _ = fs::remove_dir(&root);
    }

    #[test]
    fn different_supervisor_or_child_content_is_rejected() {
        let root = std::env::temp_dir().join(format!(
            "prefixity-frozen-binding-negative-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let supervisor = root.join("supervisor.exe");
        let child = root.join("child.exe");
        let replacement = root.join("replacement.exe");
        fs::write(&supervisor, b"supervisor bytes").unwrap();
        fs::write(&child, b"child bytes").unwrap();
        fs::write(&replacement, b"replacement bytes").unwrap();
        let expected = FrozenExecutableBinding {
            supervisor: inspect(&supervisor).unwrap(),
            child: inspect(&child).unwrap(),
        };
        let different = inspect(&replacement).unwrap();
        assert!(
            validate_frozen_executable_binding(&expected, &different, &expected.child).is_err()
        );
        assert!(
            validate_frozen_executable_binding(&expected, &expected.supervisor, &different)
                .is_err()
        );
        fs::write(&supervisor, b"rebuilt supervisor bytes").unwrap();
        let rebuilt_at_same_path = inspect(&supervisor).unwrap();
        assert!(validate_frozen_executable_binding(
            &expected,
            &rebuilt_at_same_path,
            &expected.child
        )
        .is_err());
        let _ = fs::remove_file(&replacement);
        let _ = fs::remove_file(&child);
        let _ = fs::remove_file(&supervisor);
        let _ = fs::remove_dir(&root);
    }

    #[test]
    fn missing_or_partial_preparation_binding_does_not_fallback_to_path_only() {
        assert!(
            FrozenExecutableBinding::from_implementation_fingerprints(&serde_json::json!({
                "implementation_fingerprints": {
                    "supervisor_binary": {"sha256": "a"}
                }
            }))
            .is_err()
        );
        assert!(
            FrozenExecutableBinding::from_implementation_fingerprints(&serde_json::json!({
                "implementation_fingerprints": {}
            }))
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn frozen_copy_has_its_own_object_identity_and_cannot_be_replaced() {
        let root =
            std::env::temp_dir().join(format!("prefixity-frozen-copy-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("source.exe");
        let destination = root.join("staged").join("source.exe");
        fs::write(&source, b"frozen bytes").unwrap();
        let staged = freeze_copy(&source, &destination).unwrap();
        assert_eq!(staged.file_size, source.metadata().unwrap().len());
        assert!(freeze_copy(&source, &destination).is_err());
        let _ = fs::remove_file(&destination);
        let _ = fs::remove_dir(destination.parent().unwrap());
        let _ = fs::remove_file(&source);
        let _ = fs::remove_dir(&root);
    }
}
