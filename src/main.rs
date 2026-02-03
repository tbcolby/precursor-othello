//! Othello for Precursor
//!
//! A tournament-quality Othello game with AI opponent.

#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

mod app;
mod ui;
mod menu;
mod help;
mod storage;
mod review;
mod feedback;
mod export;

use num_traits::FromPrimitive;

const SERVER_NAME: &str = "_Othello_";
const APP_NAME: &str = "Othello";

/// App opcodes for message handling
#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
enum AppOp {
    /// Redraw request from GAM
    Redraw = 0,
    /// Raw keyboard input
    Rawkeys,
    /// Focus state change
    FocusChange,
    /// AI thinking timer tick
    AiPump,
    /// Quit request
    Quit,
}

fn main() -> ! {
    // Initialize logging
    log_server::init_wait().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("Othello starting, PID {}", xous::process::id());

    // Connect to name server and register
    let xns = xous_names::XousNames::new().unwrap();
    let sid = xns
        .register_name(SERVER_NAME, None)
        .expect("can't register server");

    // Connect to GAM for graphics
    let gam = gam::Gam::new(&xns).expect("can't connect to GAM");

    // Connect to ticktimer for delays
    let ticktimer = ticktimer_server::Ticktimer::new().unwrap();

    // Register UX with GAM
    let token = gam
        .register_ux(gam::UxRegistration {
            app_name: String::from(APP_NAME),
            ux_type: gam::UxType::Chat,
            predictor: None,
            listener: sid.to_array(),
            redraw_id: AppOp::Redraw.to_u32().unwrap(),
            gotinput_id: None,
            audioframe_id: None,
            rawkeys_id: Some(AppOp::Rawkeys.to_u32().unwrap()),
            focuschange_id: Some(AppOp::FocusChange.to_u32().unwrap()),
        })
        .expect("couldn't register UX")
        .unwrap();

    // Get drawing canvas
    let content = gam.request_content_canvas(token).expect("couldn't get canvas");
    let screensize = gam.get_canvas_bounds(content).expect("couldn't get dimensions");

    log::info!(
        "Othello canvas: {}x{}",
        screensize.x,
        screensize.y
    );

    // Initialize app state
    let mut app = app::OthelloApp::new(content, screensize);

    // Load saved settings
    app.load_settings();

    // Self-connection for AI pump messages
    let self_cid = xous::connect(sid).expect("couldn't self-connect");

    // Main event loop
    let mut allow_redraw = true;

    loop {
        let msg = xous::receive_message(sid).unwrap();

        match FromPrimitive::from_usize(msg.body.id()) {
            Some(AppOp::Redraw) => {
                if allow_redraw {
                    app.draw(&gam);
                    gam.redraw().ok();
                }
            }

            Some(AppOp::Rawkeys) => xous::msg_scalar_unpack!(msg, k1, k2, k3, k4, {
                let keys = [
                    core::char::from_u32(k1 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k2 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k3 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k4 as u32).unwrap_or('\u{0000}'),
                ];

                for &key in keys.iter() {
                    if key != '\u{0000}' {
                        let should_redraw = app.handle_key(key, &gam, &ticktimer, self_cid);
                        if should_redraw && allow_redraw {
                            app.draw(&gam);
                            gam.redraw().ok();
                        }
                    }
                }
                // Check if quit was requested
                if app.should_quit {
                    break;
                }
            }),

            Some(AppOp::FocusChange) => xous::msg_scalar_unpack!(msg, new_state_code, _, _, _, {
                let new_state = gam::FocusState::convert_focus_change(new_state_code);
                match new_state {
                    gam::FocusState::Background => {
                        allow_redraw = false;
                        app.on_background();
                    }
                    gam::FocusState::Foreground => {
                        allow_redraw = true;
                        app.on_foreground();
                        app.draw(&gam);
                        gam.redraw().ok();
                    }
                }
            }),

            Some(AppOp::AiPump) => xous::msg_blocking_scalar_unpack!(msg, _, _, _, _, {
                if allow_redraw {
                    app.ai_tick(&gam, &ticktimer);
                    app.draw(&gam);
                    gam.redraw().ok();
                }
                xous::return_scalar(msg.sender, 0).ok();
            }),

            Some(AppOp::Quit) => break,

            _ => log::error!("unknown opcode: {:?}", msg),
        }
    }

    // Cleanup
    log::info!("Othello shutting down");
    xns.unregister_server(sid).unwrap();
    xous::destroy_server(sid).unwrap();
    xous::terminate_process(0)
}

use num_traits::ToPrimitive;

impl AppOp {
    fn to_u32(self) -> Option<u32> {
        num_traits::ToPrimitive::to_u32(&self)
    }
}
