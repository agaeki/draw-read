use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{KillTimer, SetTimer, TIMERPROC},
};

pub struct Timer {
    id: Option<usize>,
    func: Box<dyn FnMut()>,
    hwnd: HWND,
}

unsafe extern "system" fn callback(_param0: HWND, _param1: u32, event_id: usize, _param3: u32) {
    let timer: *mut Timer = event_id as *mut Timer;
    ((*timer).func)();
}

impl Timer {
    pub fn new(hwnd: HWND, interval_ms: u32, func: Box<dyn FnMut()>) -> Box<Self> {
        let mut timer = Box::new(Timer {
            id: None,
            func: func,
            hwnd: hwnd,
        });

        let timer_ptr: *mut Timer = &mut *timer;

        unsafe {
            let timer_id = SetTimer(
                hwnd,
                timer_ptr as usize,
                interval_ms,
                TIMERPROC::Some(callback),
            );
            timer.id = Some(timer_id);
        }

        timer
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe {
            if let Some(timer_id) = self.id {
                KillTimer(self.hwnd, timer_id);
            }
        }
    }
}

unsafe impl Sync for Timer {}
unsafe impl Send for Timer {}