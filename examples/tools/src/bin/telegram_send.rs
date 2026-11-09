//! One Telegram message (and optionally one clip) for the e2e batch recipes.
//!
//! Reads credentials the way `telegram-bot-helper` does
//! (`TelegramBot::from_env`: `TELEGRAM_BOT_TOKEN`/`TG_BOT_TOKEN` plus
//! `TELEGRAM_CHAT_ID`/`TG_CHAT_ID`, with `.env` support). Every send is
//! best-effort: without credentials or on API failure it prints a warning and
//! still exits 0, so a Telegram outage never fails the batch loop.
//!
//! Usage: `telegram_send --text "..."` and/or
//! `telegram_send --video clip.mp4 --caption "..."`.

fn send_text(text: &str) {
    telegram_bot_helper::TelegramBot::send_text_best_effort(text);
}

fn send_video(video: &str, caption: &str) {
    telegram_bot_helper::TelegramBot::send_video_best_effort(
        std::path::Path::new(video),
        &telegram_bot_helper::Caption::from(caption),
    );
}

fn main() {
    let mut args = std::env::args();
    let _program = args.next();
    let mut text: Option<String> = None;
    let mut video: Option<String> = None;
    let mut caption: Option<String> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--text" => {
                text = args.next();
            }
            "--video" => {
                video = args.next();
            }
            "--caption" => {
                caption = args.next();
            }
            unknown => {
                eprintln!("telegram_send: ignoring unknown argument: {unknown}");
            }
        }
    }
    if let Some(text) = text.as_deref() {
        send_text(text);
    }
    match (video.as_deref(), caption.as_deref()) {
        (Some(video), Some(caption)) => {
            send_video(video, caption);
        }
        (Some(_), None) => {
            eprintln!("telegram_send: --video needs --caption");
        }
        (None, _) => {}
    }
}
