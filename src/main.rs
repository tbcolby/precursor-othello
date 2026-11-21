#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

//! Othello for Precursor. Rules, AI, save format and match flow live in
//! `libs/othello-core` (host-tested); this crate is the Xous shell.

mod ai;
mod app;
mod board_view;

use num_traits::{FromPrimitive, ToPrimitive};
use precursor_appkit::{app as kit, keys, rng, store};

/// xous-names server name. Any unique string; NOT the GAM app name.
const SERVER_NAME: &str = "_Othello_";

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub(crate) enum AppOp {
    Redraw = 0,
    Rawkeys,
    FocusChange,
    /// Worker thread finished a search: (position or 255, generation, is_hint).
    AiDone,
    BasisChanged,
    Quit,
}

fn main() -> ! {
    log_server::init_wait().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("othello PID is {}", xous::process::id());

    let xns = xous_names::XousNames::new().unwrap();
    let sid = xns.register_name(SERVER_NAME, None).expect("can't register server");

    // GAM app name comes from apps/manifest.json key "othello".
    let reg = kit::register(
        &xns,
        sid,
        gam::APP_NAME_OTHELLO,
        gam::UxType::Framebuffer,
        kit::Opcodes {
            redraw: AppOp::Redraw.to_u32().unwrap(),
            rawkeys: Some(AppOp::Rawkeys.to_u32().unwrap()),
            focus_change: Some(AppOp::FocusChange.to_u32().unwrap()),
            got_input: None,
        },
    );

    let trng = rng::Trng::new(&xns).expect("TRNG unavailable");
    // Settings, statistics and the saved game are read after the PDDB mounts.
    let store = store::Store::open("othello");
    store::watch_bases(xous::connect(sid).unwrap(), AppOp::BasisChanged.to_usize().unwrap());

    let ai = ai::Worker::new(xous::connect(sid).unwrap(), AppOp::AiDone.to_usize().unwrap());
    let mut app = app::App::new(reg.screen, store, trng, ai);

    loop {
        let msg = xous::receive_message(sid).unwrap();
        match FromPrimitive::from_usize(msg.body.id()) {
            Some(AppOp::Redraw) => app.draw(),
            Some(AppOp::Rawkeys) => xous::msg_scalar_unpack!(msg, k1, k2, k3, k4, {
                for key in keys::decode_rawkeys([k1, k2, k3, k4]) {
                    if app.on_key(key) == app::Action::Leave {
                        kit::leave(app.screen());
                    }
                }
                app.draw();
            }),
            Some(AppOp::FocusChange) => xous::msg_scalar_unpack!(msg, code, _, _, _, {
                match kit::focus_state(code) {
                    gam::FocusState::Foreground => {
                        app.on_foreground();
                    }
                    gam::FocusState::Background => app.on_background(),
                }
            }),
            Some(AppOp::AiDone) => xous::msg_scalar_unpack!(msg, pos, generation, hint, _, {
                app.on_ai_done(pos, generation, hint != 0);
                app.draw();
            }),
            Some(AppOp::BasisChanged) => app.reload(),
            Some(AppOp::Quit) => break,
            None => log::warn!("unknown opcode {:?}", msg.body.id()),
        }
    }
    kit::shutdown(&xns, sid)
}
