use core::panic;
use std::collections::VecDeque;
use std::io::{self, Result};
use std::ops::Deref;
use std::slice;
use std::time::{Duration, Instant, SystemTime};
use std::{ptr::null_mut, usize};
use winapi::um::consoleapi::{
    GetConsoleMode, GetNumberOfConsoleInputEvents, ReadConsoleInputW, SetConsoleMode,
};
use winapi::um::synchapi::WaitForMultipleObjects;
use winapi::um::wincon::INPUT_RECORD;
use winapi::um::winnt::{FILE_SHARE_WRITE, GENERIC_WRITE, HANDLE};
use winapi::{
    shared::minwindef::DWORD,
    shared::winerror::WAIT_TIMEOUT,
    um::{
        fileapi::{CreateFileW, OPEN_EXISTING},
        handleapi::INVALID_HANDLE_VALUE,
        synchapi::CreateSemaphoreW,
        winbase::{INFINITE, WAIT_ABANDONED_0, WAIT_FAILED, WAIT_OBJECT_0},
        wincon::{
            COORD, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT,
            FROM_LEFT_1ST_BUTTON_PRESSED, FROM_LEFT_2ND_BUTTON_PRESSED,
            FROM_LEFT_3RD_BUTTON_PRESSED, FROM_LEFT_4TH_BUTTON_PRESSED, KEY_EVENT,
            KEY_EVENT_RECORD, MOUSE_EVENT, MOUSE_EVENT_RECORD, MOUSE_HWHEELED, MOUSE_WHEELED,
            RIGHTMOST_BUTTON_PRESSED,
        },
        winnt::{FILE_SHARE_READ, GENERIC_READ},
    },
};

fn main() {
    println!("Testing");
    let _stdout = io::stdout();

    let _raw = set_raw_mode_console();

    let seconds = Duration::from_secs(10);
    let start = SystemTime::now();
    loop {
        match read_char() {
            Ok('q') => {
                break;
            }
            Ok(ch) => {
                println!("{}", ch)
            }
            _ => {}
        };
        match start.elapsed() {
            Ok(elapsed) if elapsed > seconds => {
                break;
            }
            _ => (),
        }
    }
    let _ = unset_raw_mode_console();
}

fn read_char() -> Result<char> {
    if let Ok(InputRecord::KeyEvent(KeyEvent {
        key_down: _,
        repeat_count: _,
        virtual_key_code: _,
        virtual_scan_code: _,
        u_char: c2,
        control_key_state: _,
    })) = read()
    {
        let ch = std::char::from_u32(c2 as u32).unwrap();
        return Ok(ch);
    }
    Ok('u')
}

fn read() -> Result<InputRecord> {
    match read_int()? {
        InputRecord::KeyEvent(event) => Ok(InputRecord::KeyEvent(event)),
        _ => unreachable!(),
    }
}
fn read_int() -> Result<InputRecord> {
    let mut reader = InternalEventReader::default();
    reader.read()
}

pub struct InternalEventReader {
    events: VecDeque<InputRecord>,
    source: Option<Box<WindowsEventSource>>,
    skipped_events: Vec<InputRecord>,
}

impl InternalEventReader {
    pub fn read(&mut self) -> Result<InputRecord> {
        let mut skipped_events = VecDeque::new();
        loop {
            while let Some(event) = self.events.pop_front() {
                while let Some(event) = skipped_events.pop_front() {
                    self.events.push_back(event)
                }
                return Ok(event);
            }
            let _ = self.poll()?;
        }
    }

    pub fn poll(&mut self) -> Result<bool> {
        for _event in &self.events {
            return Ok(true);
        }

        let event_source = match self.source.as_mut() {
            Some(source) => source,
            None => return Err(io::Error::new(io::ErrorKind::Other, "Something went wrong")),
        };
        let poll_timeout = PollTimeout::new(None);
        loop {
            let maybe_event = match event_source.try_read(poll_timeout.leftover()) {
                Ok(None) => None,
                Ok(Some(event)) => Some(event),
                Err(e) => {
                    if e.kind() == io::ErrorKind::Interrupted {
                        return Ok(false);
                    }

                    return Err(e);
                }
            };

            if poll_timeout.elapsed() || maybe_event.is_some() {
                self.events.extend(self.skipped_events.drain(..));

                if let Some(event) = maybe_event {
                    self.events.push_front(event);
                    return Ok(true);
                }

                return Ok(false);
            }
        }
    }
}
impl Default for InternalEventReader {
    fn default() -> Self {
        let source = WindowsEventSource::new();
        let source = source.ok().map(|x| Box::new(x) as Box<WindowsEventSource>);
        Self {
            source,
            events: VecDeque::with_capacity(32),
            skipped_events: Vec::with_capacity(32),
        }
    }
}
pub struct PollTimeout {
    timeout: Option<Duration>,
    start: Instant,
}

impl PollTimeout {
    /// Constructs a new `PollTimeout` with the given optional `Duration`.
    pub fn new(timeout: Option<Duration>) -> PollTimeout {
        PollTimeout {
            timeout,
            start: Instant::now(),
        }
    }

    /// Returns whether the timeout has elapsed.
    ///
    /// It always returns `false` if the initial timeout was set to `None`.
    pub fn elapsed(&self) -> bool {
        self.timeout
            .map(|timeout| self.start.elapsed() >= timeout)
            .unwrap_or(false)
    }

    /// Returns the timeout leftover (initial timeout duration - elapsed duration).
    pub fn leftover(&self) -> Option<Duration> {
        self.timeout.map(|timeout| {
            let elapsed = self.start.elapsed();

            if elapsed >= timeout {
                Duration::from_secs(0)
            } else {
                timeout - elapsed
            }
        })
    }
}
pub struct WindowsEventSource {
    console: ConsoleMode,
    poll: WinApiPoll,
    surrogate_buffer: Option<u16>,
}
impl WindowsEventSource {
    pub fn new() -> Result<Self> {
        let console = get_current_console_mode()?;
        Ok(WindowsEventSource {
            console,
            poll: WinApiPoll::new()?,
            surrogate_buffer: None,
        })
    }
}
impl EventSource for WindowsEventSource {
    fn try_read(&mut self, timeout: Option<Duration>) -> std::io::Result<Option<InputRecord>> {
        let poll_timeout = PollTimeout::new(timeout);

        loop {
            if let Some(event_ready) = self.poll.poll(poll_timeout.leftover())? {
                let number = self.console.number_of_console_input_events()?;
                if event_ready && number != 0 {
                    let event = Some(self.console.read_single_input_event()?);

                    if let Some(event) = event {
                        return Ok(Some(event));
                    }
                }
            }

            if poll_timeout.elapsed() {
                return Ok(None);
            }
        }
    }
}
pub(crate) trait EventSource {
    fn try_read(&mut self, timeout: Option<Duration>) -> io::Result<Option<InputRecord>>;
}
pub struct WinApiPoll {
    waker: Handle,
}

impl WinApiPoll {
    pub fn new() -> Result<Self> {
        Ok(Self {
            waker: Handle::new()?,
        })
    }

    pub fn poll(&mut self, timeout: Option<Duration>) -> Result<Option<bool>> {
        let dw_millis = if let Some(duration) = timeout {
            duration.as_millis() as u32
        } else {
            INFINITE
        };

        let console_handle = get_current_handle()?;
        let handles = &[*console_handle];

        let output =
            unsafe { WaitForMultipleObjects(handles.len() as u32, handles.as_ptr(), 0, dw_millis) };

        match output {
            output if output == WAIT_OBJECT_0 => {
                // input handle triggered
                Ok(Some(true))
            }
            WAIT_TIMEOUT | WAIT_ABANDONED_0 => {
                // timeout elapsed
                Ok(None)
            }
            WAIT_FAILED => Err(io::Error::last_os_error()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "WaitForMultipleObjects returned unexpected result.",
            )),
        }
    }
}

impl Handle {
    fn new() -> Result<Self> {
        let handle = unsafe { CreateSemaphoreW(std::ptr::null_mut(), 0, 1, std::ptr::null_mut()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let handle = Handle { handle };
        Ok(handle)
    }
}
//Wrapper for COORD to strip away the junk
pub struct Coordinate {
    x: i16,
    y: i16,
}

impl From<COORD> for Coordinate {
    fn from(value: COORD) -> Self {
        Coordinate {
            x: value.X,
            y: value.Y,
        }
    }
}

//Controll key state
//i.e. Shift, Ctrl, Alt, Caps Lock
//https://learn.microsoft.com/en-us/dotnet/api/system.management.automation.host.controlkeystates?view=powershellsdk-7.4.0
pub struct ControlKey {
    state: u32,
}

impl From<u32> for ControlKey {
    fn from(value: u32) -> Self {
        ControlKey { state: value }
    }
}

pub enum ControlKeyStateTypes {
    CapsLock = 0x0080,
    NumLock = 0x0020,
    ScrollLock = 0x0040,
    Shift = 0x0010,
    LeftAlt = 0x0002,
    LeftCtrl = 0x0008,
    RightAlt = 0x0001,
    RightCtrl = 0x0004,
}

impl ControlKey {
    pub fn has_state(&self, state: u32) -> bool {
        (state & self.state) != 0
    }
}

//Wrapper for Keyboard button events
//https://learn.microsoft.com/en-us/windows/console/key-event-record-str
pub struct KeyEvent {
    pub key_down: bool,
    pub repeat_count: u16,
    pub virtual_key_code: u16,
    pub virtual_scan_code: u16,
    pub u_char: u16,
    pub control_key_state: ControlKey,
}

impl From<&KEY_EVENT_RECORD> for KeyEvent {
    fn from(value: &KEY_EVENT_RECORD) -> Self {
        KeyEvent {
            key_down: value.bKeyDown != 0,
            repeat_count: value.wRepeatCount,
            virtual_key_code: value.wVirtualKeyCode,
            virtual_scan_code: value.wVirtualScanCode,
            u_char: unsafe { *value.uChar.UnicodeChar() },
            control_key_state: value.dwControlKeyState.into(),
        }
    }
}

//wrapper for mouse events
//https://learn.microsoft.com/en-us/windows/console/mouse-event-record-str
pub struct MouseEvent {
    pub mouse_pos: Coordinate,
    pub button_state: MouseButton,
    pub control_key_state: ControlKey,
    pub event_flags: MouseEventType,
}

impl From<MOUSE_EVENT_RECORD> for MouseEvent {
    fn from(event: MOUSE_EVENT_RECORD) -> Self {
        MouseEvent {
            mouse_pos: event.dwMousePosition.into(),
            button_state: event.dwButtonState.into(),
            control_key_state: event.dwControlKeyState.into(),
            event_flags: event.dwEventFlags.into(),
        }
    }
}

pub enum MouseEventType {
    Release = 0x0000,
    Move = 0x0001,
    DoubleClick = 0x0002,
    Scroll = 0x0004,
    Unknown = 0x0099,
}

impl From<DWORD> for MouseEventType {
    fn from(value: DWORD) -> Self {
        match value {
            0x0000 => MouseEventType::Release,
            0x0001 => MouseEventType::Move,
            0x0002 => MouseEventType::DoubleClick,
            0x0004 => MouseEventType::Scroll,
            _ => MouseEventType::Unknown,
        }
    }
}

pub struct MouseButton {
    state: i32,
}

impl From<DWORD> for MouseButton {
    fn from(event: DWORD) -> Self {
        let state = event as i32;
        MouseButton { state }
    }
}

pub enum MouseButtonOptions {
    LeftFirst,
    LeftSecond,
    LeftThird,
    LeftFourth,
    RightMost,
    Unknown,
}

pub enum VerticalScrollDirection {
    Up,
    Down,
    Still,
}

pub enum HorizontalScrollDirection {
    Right,
    Left,
    Still,
}

impl MouseButton {
    pub fn nothing_held(&self) -> bool {
        self.state == 0
    }
    pub fn check_button(&self, left_num: MouseButtonOptions) -> bool {
        let button: DWORD = match left_num {
            MouseButtonOptions::LeftFirst => FROM_LEFT_1ST_BUTTON_PRESSED,
            MouseButtonOptions::LeftSecond => FROM_LEFT_2ND_BUTTON_PRESSED,
            MouseButtonOptions::LeftThird => FROM_LEFT_3RD_BUTTON_PRESSED,
            MouseButtonOptions::LeftFourth => FROM_LEFT_4TH_BUTTON_PRESSED,
            MouseButtonOptions::RightMost => RIGHTMOST_BUTTON_PRESSED,
            _ => 0x0000,
        };
        self.state as u32 & button != 0
    }
    pub fn vert_scroll_direction(&self) -> VerticalScrollDirection {
        match (self.state as u32 & MOUSE_WHEELED).cmp(&0) {
            std::cmp::Ordering::Less => VerticalScrollDirection::Down,
            std::cmp::Ordering::Greater => VerticalScrollDirection::Up,
            std::cmp::Ordering::Equal => VerticalScrollDirection::Still,
        }
    }
    pub fn hor_scroll_direction(&self) -> HorizontalScrollDirection {
        match (self.state as u32 & MOUSE_HWHEELED).cmp(&0) {
            std::cmp::Ordering::Less => HorizontalScrollDirection::Left,
            std::cmp::Ordering::Greater => HorizontalScrollDirection::Right,
            std::cmp::Ordering::Equal => HorizontalScrollDirection::Still,
        }
    }
    pub fn raw(&self) -> i32 {
        self.state
    }
}

//Input record
pub enum InputRecord {
    KeyEvent(KeyEvent),
    MouseEvent(MouseEvent),
    UnknownEvent,
    //WindowBufferSizeEvent(WindowBufferSizeEvent),
    //FocusEvent(FocusEvent),
    //MenuEvent(MenuEvent),
}

impl From<INPUT_RECORD> for InputRecord {
    fn from(value: INPUT_RECORD) -> Self {
        match value.EventType {
            KEY_EVENT => InputRecord::KeyEvent(KeyEvent::from(unsafe { value.Event.KeyEvent() })),
            MOUSE_EVENT => InputRecord::MouseEvent(unsafe { *value.Event.MouseEvent() }.into()),
            _ => Self::UnknownEvent,
        }
    }
}

//Set and Read the console mode
pub struct ConsoleMode {
    handle: Handle,
}

impl From<HANDLE> for ConsoleMode {
    fn from(value: HANDLE) -> Self {
        ConsoleMode {
            handle: Handle { handle: value },
        }
    }
}

impl ConsoleMode {
    //https://docs.microsoft.com/en-us/windows/console/setconsolemode
    pub fn set_mode(&self, mode: DWORD) -> Result<()> {
        let handle: HANDLE = *self.handle;
        if unsafe { SetConsoleMode(handle, mode) } == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    //https://docs.microsoft.com/en-us/windows/console/getconsolemode
    pub fn current_mode(&self) -> Result<u32> {
        let mut console_mode: u32 = 0;
        let handle: HANDLE = *self.handle;
        unsafe { GetConsoleMode(handle, &mut console_mode) };
        if console_mode == 0 {
            panic!("Should not be 0")
        }
        Ok(console_mode)
    }

    fn read_input(&self, buf: &mut [INPUT_RECORD]) -> Result<usize> {
        let mut num_records = 0;
        debug_assert!(buf.len() < u32::MAX as usize);

        unsafe {
            let res = ReadConsoleInputW(
                *self.handle,
                buf.as_mut_ptr(),
                buf.len() as u32,
                &mut num_records,
            );
            if res == 0 {
                panic!("Something went wrong")
            }
        }

        Ok(num_records as usize)
    }

    pub fn read_single_input_event(&self) -> Result<InputRecord> {
        let mut record: INPUT_RECORD = unsafe { core::mem::zeroed() };

        {
            let buf = slice::from_mut(&mut record);
            let num_read = self.read_input(buf)?;

            debug_assert!(num_read == 1);
        }

        Ok(record.into())
    }

    pub fn number_of_console_input_events(&self) -> Result<u32> {
        let mut buf_len: DWORD = 0;
        unsafe { GetNumberOfConsoleInputEvents(*self.handle, &mut buf_len) };
        Ok(buf_len)
    }
}

pub struct Handle {
    handle: HANDLE,
}

impl Deref for Handle {
    type Target = HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

const NOT_RAW_MODE: DWORD = ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT;

fn set_raw_mode_console() -> std::io::Result<()> {
    let console_mode = get_current_console_mode()?;

    let dw_mode = console_mode.current_mode()?;

    let new_mode = dw_mode & !NOT_RAW_MODE;

    console_mode.set_mode(new_mode)?;

    Ok(())
}

fn unset_raw_mode_console() -> std::io::Result<()> {
    let console_mode = get_current_console_mode()?;

    let dw_mode = console_mode.current_mode()?;

    let new_mode = dw_mode | NOT_RAW_MODE;

    console_mode.set_mode(new_mode)?;

    Ok(())
}

fn get_current_console_mode() -> Result<ConsoleMode> {
    let utf16: Vec<u16> = "CONIN$\0".encode_utf16().collect();
    let utf16_ptr: *const u16 = utf16.as_ptr();

    let handle: HANDLE;
    unsafe {
        //https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
        handle = CreateFileW(
            utf16_ptr,
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE {
            panic!("INVALID Handle");
        }
    }
    Ok(ConsoleMode::from(handle))
}

fn get_current_handle() -> Result<Handle> {
    let utf16: Vec<u16> = "CONIN$\0".encode_utf16().collect();
    let utf16_ptr: *const u16 = utf16.as_ptr();

    let handle: HANDLE;
    unsafe {
        //https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
        handle = CreateFileW(
            utf16_ptr,
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE {
            panic!("INVALID Handle");
        }
    }
    return Ok(Handle { handle });
}
