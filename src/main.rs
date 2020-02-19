extern crate gstreamer as gst;
extern crate gstreamer_player as gst_player;
extern crate glib;

use glib::Cast;

use std::env;
use std::sync::{Arc, Mutex};

// TODO:
// 1. frame-by-frame check for fullwhites (watch out for timer)
// 2. display ~5 second snippets to verify fullwhites
// 3. allow users to speed through verification or skip it entirely

// probably will be used for human verification of timing
fn main_loop(uri: &str) {
    // from gstreamer-rs/examples/player
    let main_loop = glib::MainLoop::new(None, false);

    let dispatcher = gst_player::PlayerGMainContextSignalDispatcher::new(None);
    let player = gst_player::Player::new(
        None,
        Some(&dispatcher.upcast::<gst_player::PlayerSignalDispatcher>()),
    );

    player.set_uri(uri);
    println!("set uri to {}", uri);

    let lock = Arc::new(Mutex::new(Ok(())));

    let main_loop_clone = main_loop.clone();
    // Connect to the player's "end-of-stream" signal, which tells us
    // when the currently played media stream ends.
    player.connect_end_of_stream(move |player| {
        println!("info: end of stream encountered");
        let main_loop = &main_loop_clone;
        player.stop();
        main_loop.quit();
    });

    let main_loop_clone = main_loop.clone();
    let lock_clone = lock.clone();
    player.connect_error(move |player, err| {
        println!("info: error occurred in playback:");
        println!("{}", err);
        let main_loop = &main_loop_clone;
        let lock = &lock_clone;

        *lock.lock().unwrap() = Err(err.clone());

        player.stop();
        main_loop.quit();
    });

    player.play();

    println!("starting mainloop");

    main_loop.run();
    // let _guard = lock.as_ref().lock().unwrap();
    // guard.clone().map_err(|e| e.into());
}


fn main() {
    println!("starting timer");

    let args: Vec<_> = env::args().collect();
    let uri: &str = if args.len() == 2 {
        args[1].as_ref()
    } else {
        eprintln!("usage: h2a-timer <filename>");
        std::process::exit(-1);
    };

    println!("got arg: {}", uri);

    println!("starting gstreamer");

    let res = gst::init();
    match res {
        Ok(_v) => println!("successfully initialized gstreamer"),
        Err(e) => eprintln!("!!! error initializing gstreamer: {}", e),
    }

    let mut filename = String::from("file:///");
    filename.push_str(uri);

    main_loop(filename.as_ref());

    println!("mainloop exiting");
}
