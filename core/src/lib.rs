//! State, caching, and typed Villain services for the Qt Quick shell.

use std::{
    ffi::{CString, OsStr},
    io::{BufRead, BufReader, Write},
    os::{raw::c_char, unix::net::UnixStream},
    path::PathBuf,
};

use base64::Engine;
use serde::{Deserialize, Serialize};

const REQUIRED_PROTOCOL_VERSION: u32 = 2;
const PREVIEW_WIDTH: u32 = 480;
const PREVIEW_HEIGHT: u32 = 270;
const EMPTY_C_STRING: &[u8] = b"\0";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
struct WindowId(u64);

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct WireWindow {
    id: WindowId,
    title: String,
    app_id: String,
    workspace: usize,
    minimized: bool,
    #[serde(default)]
    floating: bool,
    #[serde(default)]
    fullscreen: bool,
    focused: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct WireWorkspace {
    workspace: usize,
    active: bool,
    window_count: usize,
    visible_window_count: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct WorkspacePreview {
    workspace: usize,
    width: u32,
    height: u32,
    png_base64: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
enum DispatchRequest {
    FocusWorkspace { workspace: usize },
    FocusWindow { window: WindowId },
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "query", rename_all = "kebab-case")]
enum Query {
    Windows,
    Workspaces,
    WorkspacePreview {
        workspace: usize,
        width: u32,
        height: u32,
    },
    Version,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
enum Request {
    Dispatch(DispatchRequest),
    Query(Query),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
enum Response {
    Ok,
    Windows(Vec<WireWindow>),
    Workspaces(Vec<WireWorkspace>),
    WorkspacePreview(WorkspacePreview),
    Version { protocol: u32, villain: String },
    Error { message: String },
}

struct Client {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
}

impl Client {
    fn connect() -> Result<Self, String> {
        let stream = UnixStream::connect(socket_path()?).map_err(|error| error.to_string())?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(2)))
            .map_err(|error| error.to_string())?;
        Ok(Self {
            reader: BufReader::new(stream.try_clone().map_err(|error| error.to_string())?),
            writer: stream,
        })
    }

    fn request(&mut self, request: &Request) -> Result<Response, String> {
        serde_json::to_writer(&mut self.writer, request).map_err(|error| error.to_string())?;
        self.writer
            .write_all(b"\n")
            .and_then(|()| self.writer.flush())
            .map_err(|error| error.to_string())?;
        let mut line = String::new();
        if self
            .reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?
            == 0
        {
            return Err("Villain closed the IPC connection".into());
        }
        serde_json::from_str(&line).map_err(|error| error.to_string())
    }
}

fn socket_path() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("VILLAIN_SOCKET") {
        return Ok(path.into());
    }
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .ok_or_else(|| "XDG_RUNTIME_DIR is not set".to_string())?;
    let display = std::env::var_os("WAYLAND_DISPLAY")
        .ok_or_else(|| "WAYLAND_DISPLAY is not set".to_string())?;
    Ok(socket_path_for_display(runtime.as_ref(), display.as_ref()))
}

fn socket_path_for_display(runtime: &OsStr, display: &OsStr) -> PathBuf {
    let display = display.to_string_lossy().replace(['/', '\\'], "_");
    PathBuf::from(runtime).join(format!("villain-{display}.sock"))
}

struct WindowState {
    wire: WireWindow,
    title: CString,
    app_id: CString,
}

impl WindowState {
    fn new(wire: WireWindow) -> Self {
        let title = sanitized_c_string(&wire.title);
        let app_id = sanitized_c_string(&wire.app_id);
        Self {
            wire,
            title,
            app_id,
        }
    }
}

fn sanitized_c_string(value: &str) -> CString {
    CString::new(value.replace('\0', "�")).expect("replacement removes NUL bytes")
}

/// Opaque service state exposed through the stable C header.
pub struct KsCore {
    client: Option<Client>,
    connected: bool,
    error: CString,
    workspaces: Vec<WireWorkspace>,
    windows: Vec<WindowState>,
    previews: Vec<Vec<u8>>,
    preview_revision: u64,
}

impl Default for KsCore {
    fn default() -> Self {
        Self {
            client: None,
            connected: false,
            error: sanitized_c_string("Waiting for Villain"),
            workspaces: Vec::new(),
            windows: Vec::new(),
            previews: vec![Vec::new(); 10],
            preview_revision: 0,
        }
    }
}

impl KsCore {
    fn ensure_client(&mut self) -> Result<(), String> {
        if self.client.is_some() {
            return Ok(());
        }
        let mut client = Client::connect()?;
        match client.request(&Request::Query(Query::Version))? {
            Response::Version { protocol, .. } if protocol >= REQUIRED_PROTOCOL_VERSION => {
                self.client = Some(client);
                Ok(())
            }
            Response::Version { protocol, .. } => Err(format!(
                "Villain IPC protocol {protocol} is too old; Knave Shell requires {REQUIRED_PROTOCOL_VERSION}"
            )),
            response => Err(unexpected("version", response)),
        }
    }

    fn request(&mut self, request: Request) -> Result<Response, String> {
        self.ensure_client()?;
        let result = self.client.as_mut().unwrap().request(&request);
        if result.is_err() {
            self.client = None;
        }
        result
    }

    fn refresh(&mut self) -> Result<(), String> {
        let workspaces = match self.request(Request::Query(Query::Workspaces))? {
            Response::Workspaces(value) => value,
            response => return Err(unexpected("workspaces", response)),
        };
        let windows = match self.request(Request::Query(Query::Windows))? {
            Response::Windows(value) => value,
            response => return Err(unexpected("windows", response)),
        };
        self.workspaces = workspaces;
        self.windows = windows.into_iter().map(WindowState::new).collect();
        self.connected = true;
        self.error = sanitized_c_string("");

        let active = self
            .workspaces
            .iter()
            .find(|workspace| workspace.active)
            .map(|workspace| workspace.workspace);
        let initial = self
            .workspaces
            .iter()
            .filter(|workspace| workspace.window_count > 0)
            .map(|workspace| workspace.workspace)
            .find(|workspace| {
                self.previews
                    .get(workspace.saturating_sub(1))
                    .is_some_and(Vec::is_empty)
            });
        let target = initial.or(active);
        if let Some(workspace) = target {
            self.refresh_preview(workspace)?;
        }
        Ok(())
    }

    fn refresh_preview(&mut self, workspace: usize) -> Result<(), String> {
        let response = self.request(Request::Query(Query::WorkspacePreview {
            workspace,
            width: PREVIEW_WIDTH,
            height: PREVIEW_HEIGHT,
        }))?;
        let preview = match response {
            Response::WorkspacePreview(value) => value,
            response => return Err(unexpected("workspace preview", response)),
        };
        if preview.workspace != workspace
            || preview.width != PREVIEW_WIDTH
            || preview.height != PREVIEW_HEIGHT
        {
            return Err("Villain returned mismatched preview metadata".into());
        }
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(preview.png_base64)
            .map_err(|error| format!("invalid preview encoding: {error}"))?;
        if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err("Villain returned a preview that is not a PNG".into());
        }
        if let Some(slot) = self.previews.get_mut(workspace.saturating_sub(1)) {
            *slot = bytes;
            self.preview_revision = self.preview_revision.wrapping_add(1);
        }
        Ok(())
    }

    fn dispatch(&mut self, request: DispatchRequest) -> Result<(), String> {
        match self.request(Request::Dispatch(request))? {
            Response::Ok => Ok(()),
            Response::Error { message } => Err(message),
            response => Err(unexpected("dispatch", response)),
        }
    }

    fn fail(&mut self, error: String) {
        self.connected = false;
        self.error = sanitized_c_string(&error);
    }
}

fn unexpected(expected: &str, response: Response) -> String {
    match response {
        Response::Error { message } => message,
        _ => format!("Villain returned an unexpected response for {expected}"),
    }
}

fn core_ref<'a>(core: *const KsCore) -> Option<&'a KsCore> {
    // SAFETY: Every public accessor treats a null handle as absent. Non-null
    // handles originate from `ks_core_new` and remain owned by the caller.
    unsafe { core.as_ref() }
}

fn core_mut<'a>(core: *mut KsCore) -> Option<&'a mut KsCore> {
    // SAFETY: Mutating functions require the caller to serialize access to its
    // opaque handle, which the Qt adapter does on the GUI thread.
    unsafe { core.as_mut() }
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_new() -> *mut KsCore {
    Box::into_raw(Box::new(KsCore::default()))
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `core` must be null or a live handle returned by `ks_core_new` that has not
/// previously been freed.
pub unsafe extern "C" fn ks_core_free(core: *mut KsCore) {
    if !core.is_null() {
        // SAFETY: The pointer was returned by `Box::into_raw` in `ks_core_new`
        // and this function consumes it exactly once.
        unsafe { drop(Box::from_raw(core)) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_refresh(core: *mut KsCore) -> bool {
    let Some(core) = core_mut(core) else {
        return false;
    };
    match core.refresh() {
        Ok(()) => true,
        Err(error) => {
            core.fail(error);
            false
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_connected(core: *const KsCore) -> bool {
    core_ref(core).is_some_and(|core| core.connected)
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_last_error(core: *const KsCore) -> *const c_char {
    core_ref(core)
        .map(|core| core.error.as_ptr())
        .unwrap_or(EMPTY_C_STRING.as_ptr().cast())
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_preview_revision(core: *const KsCore) -> u64 {
    core_ref(core).map_or(0, |core| core.preview_revision)
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_workspace_count(core: *const KsCore) -> usize {
    core_ref(core).map_or(0, |core| core.workspaces.len())
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_workspace_number(core: *const KsCore, index: usize) -> u32 {
    core_ref(core)
        .and_then(|core| core.workspaces.get(index))
        .map_or(0, |workspace| workspace.workspace as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_workspace_active(core: *const KsCore, index: usize) -> bool {
    core_ref(core)
        .and_then(|core| core.workspaces.get(index))
        .is_some_and(|workspace| workspace.active)
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_workspace_window_count(core: *const KsCore, index: usize) -> usize {
    core_ref(core)
        .and_then(|core| core.workspaces.get(index))
        .map_or(0, |workspace| workspace.window_count)
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_workspace_visible_window_count(
    core: *const KsCore,
    index: usize,
) -> usize {
    core_ref(core)
        .and_then(|core| core.workspaces.get(index))
        .map_or(0, |workspace| workspace.visible_window_count)
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_window_count(core: *const KsCore) -> usize {
    core_ref(core).map_or(0, |core| core.windows.len())
}

fn window(core: *const KsCore, index: usize) -> Option<&'static WindowState> {
    // The returned pointer is only used by immediate FFI accessors and remains
    // valid until the next mutable operation on the same core handle.
    core_ref(core).and_then(|core| core.windows.get(index))
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_window_id(core: *const KsCore, index: usize) -> u64 {
    window(core, index).map_or(0, |window| window.wire.id.0)
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_window_title(core: *const KsCore, index: usize) -> *const c_char {
    window(core, index)
        .map(|window| window.title.as_ptr())
        .unwrap_or(EMPTY_C_STRING.as_ptr().cast())
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_window_app_id(core: *const KsCore, index: usize) -> *const c_char {
    window(core, index)
        .map(|window| window.app_id.as_ptr())
        .unwrap_or(EMPTY_C_STRING.as_ptr().cast())
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_window_workspace(core: *const KsCore, index: usize) -> u32 {
    window(core, index).map_or(0, |window| window.wire.workspace as u32)
}

macro_rules! window_bool {
    ($name:ident, $field:ident) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(core: *const KsCore, index: usize) -> bool {
            window(core, index).is_some_and(|window| window.wire.$field)
        }
    };
}

window_bool!(ks_core_window_minimized, minimized);
window_bool!(ks_core_window_floating, floating);
window_bool!(ks_core_window_fullscreen, fullscreen);
window_bool!(ks_core_window_focused, focused);

#[unsafe(no_mangle)]
/// # Safety
///
/// When non-null, `length` must point to writable storage for one `size_t`.
pub unsafe extern "C" fn ks_core_preview_png(
    core: *const KsCore,
    workspace: u32,
    length: *mut usize,
) -> *const u8 {
    if !length.is_null() {
        // SAFETY: The caller provided writable storage for one `size_t`.
        unsafe { *length = 0 };
    }
    let Some(bytes) = workspace
        .checked_sub(1)
        .and_then(|index| core_ref(core)?.previews.get(index as usize))
        .filter(|bytes| !bytes.is_empty())
    else {
        return std::ptr::null();
    };
    if !length.is_null() {
        // SAFETY: Same pointer validated above; bytes remain owned by `core`.
        unsafe { *length = bytes.len() };
    }
    bytes.as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_focus_workspace(core: *mut KsCore, workspace: u32) -> bool {
    let Some(core) = core_mut(core) else {
        return false;
    };
    match core.dispatch(DispatchRequest::FocusWorkspace {
        workspace: workspace as usize,
    }) {
        Ok(()) => true,
        Err(error) => {
            core.fail(error);
            false
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ks_core_focus_window(core: *mut KsCore, window: u64) -> bool {
    let Some(core) = core_mut(core) else {
        return false;
    };
    match core.dispatch(DispatchRequest::FocusWindow {
        window: WindowId(window),
    }) {
        Ok(()) => true,
        Err(error) => {
            core.fail(error);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_requests_match_villain_protocol_v2() {
        assert_eq!(
            serde_json::to_string(&Request::Query(Query::WorkspacePreview {
                workspace: 2,
                width: 480,
                height: 270,
            }))
            .unwrap(),
            r#"{"type":"query","payload":{"query":"workspace-preview","workspace":2,"width":480,"height":270}}"#
        );
        assert_eq!(
            serde_json::to_string(&Request::Dispatch(DispatchRequest::FocusWindow {
                window: WindowId(7),
            }))
            .unwrap(),
            r#"{"type":"dispatch","payload":{"action":"focus-window","window":7}}"#
        );
    }

    #[test]
    fn socket_path_is_display_scoped_and_cannot_escape() {
        assert_eq!(
            socket_path_for_display(OsStr::new("/tmp/runtime"), OsStr::new("../wayland-2")),
            PathBuf::from("/tmp/runtime/villain-.._wayland-2.sock")
        );
    }

    #[test]
    fn ffi_strings_replace_interior_nul() {
        let value = sanitized_c_string("one\0two");
        assert_eq!(value.as_c_str(), c"one�two");
    }
}
