use crate::nwg::MousePressEvent;
use crate::nwg::WindowFlags;
use std::rc::Rc;
use std::time::Duration;

use crate::app_logic::App;

use core::cell::RefCell;

pub struct BasicAppUi {
    inner: Rc<RefCell<App>>,
    main_default_handler: RefCell<Option<nwg::EventHandler>>,
    screenshot_default_handler: RefCell<Option<nwg::EventHandler>>,
}

impl nwg::NativeUi<BasicAppUi> for App {
    fn build_ui(mut data: App) -> Result<BasicAppUi, nwg::NwgError> {
        use nwg::Event as E;

        // Controls
        nwg::Window::builder()
            .ex_flags(0)
            .topmost(true)
            .size((85, 45))
            .position((300, 300))
            .title("")
            .build(&mut data.window)?;

        nwg::Button::builder()
            .text("Read")
            .position((0, 0))
            .parent(&data.window)
            .build(&mut data.read_button)?;

        nwg::Window::builder()
            .flags(WindowFlags::POPUP)
            .ex_flags(0)
            .topmost(true)
            .title("")
            .parent(Some(&data.window))
            .build(&mut data.screenshot_window)?;

        nwg::ImageFrame::builder()
            .parent(&data.screenshot_window)
            .build(&mut data.screenshot_frame)?;

        #[allow(deprecated)]
        let _ = nwg::Timer::builder()
            .parent(&data.window)
            .interval(1000)
            .stopped(false)
            .build(&mut data.timer);

        // Wrap-up
        let ui = BasicAppUi {
            inner: Rc::new(RefCell::new(data)),
            main_default_handler: Default::default(),
            screenshot_default_handler: Default::default(),
        };

        // Events
        let evt_ui = Rc::downgrade(&ui.inner);
        let screenshot_evt_ui = Rc::downgrade(&ui.inner);
        let handle_events = move |evt, _evt_data, _handle| {
            if let Some(app) = evt_ui.upgrade() {
                match evt {
                    E::OnButtonClick => {
                        // hide window
                        app.borrow().window.set_visible(false);
                        std::thread::sleep(Duration::from_millis(200));

                        // take screenshot
                        app.borrow_mut().refresh_screenshot();

                        // show screenshot window
                        app.borrow().screenshot_window.set_visible(true);
                    }
                    E::OnWindowClose => {
                        nwg::stop_thread_dispatch();
                    }
                    E::OnTimerTick => {
                        //println!("Timer ticked for {:?} {:?}", handle, ui.window.handle);
                        /*let new_position = update_and_get_new_position(
                            &mut ui.engine.borrow_mut(),
                            &mut ui.tts.borrow_mut(),
                            &mut ui.read_position.borrow_mut(),
                            &mut ui.to_read.borrow_mut(),
                        );

                        ui.window
                            .set_position(new_position.0 as i32, new_position.1 as i32);*/
                    }
                    _ => {}
                }
            }
        };

        let handle_screenshot_events = move |evt, _evt_data, _handle| {
            if let Some(app) = screenshot_evt_ui.upgrade() {
                match evt {
                    E::OnMousePress(MousePressEvent::MousePressLeftDown) => {
                        println!("OnMousePress MousePressLeftDown");
                        let mouse_pos = nwg::GlobalCursor::local_position(
                            app.borrow().screenshot_frame.handle,
                            None,
                        );
                        app.borrow_mut().start_draw_rect(mouse_pos);
                    }
                    E::OnMousePress(MousePressEvent::MousePressLeftUp) => {
                        println!("OnMousePress MousePressLeftUp");
                        app.borrow_mut().end_draw_rect();
                        // hide window
                        app.borrow().screenshot_window.set_visible(false);
                        // hide window
                        app.borrow().window.set_visible(true);
                    }
                    E::OnPaint => {
                        println!("OnPaint");
                        if let Ok(mut mut_app) = app.try_borrow_mut() {
                            mut_app.draw_rect();
                        }
                    }
                    E::OnMouseMove => {
                        println!("OnMouseMove");
                        app.borrow_mut().draw_rect();
                    }
                    _ => {}
                }
            }
        };

        *ui.main_default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
            &ui.inner.borrow().window.handle,
            handle_events,
        ));
        *ui.screenshot_default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
            &ui.inner.borrow().screenshot_window.handle,
            handle_screenshot_events,
        ));
        return Ok(ui);
    }
}

impl Drop for BasicAppUi {
    /// To make sure that everything is freed without issues, the default handler must be unbound.
    fn drop(&mut self) {
        let main_handler = self.main_default_handler.borrow();
        if main_handler.is_some() {
            nwg::unbind_event_handler(main_handler.as_ref().unwrap());
        }

        let screenshot_handler = self.screenshot_default_handler.borrow();
        if screenshot_handler.is_some() {
            nwg::unbind_event_handler(screenshot_handler.as_ref().unwrap());
        }
    }
}
