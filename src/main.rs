//! `cutcraft-borders`: PowerWhittle runs it behind its Generate Border form.
//!
//! ```text
//! cutcraft-borders picture STYLE RATIO DETAILS OUT   one repeat of a moulding, as a picture
//! cutcraft-borders depth PICTURE RATIO OUT           that picture's depth map (white highest)
//! cutcraft-borders styles                            the styles, one "id<TAB>name" a line
//! ```
//!
//! The key comes in the `CUTCRAFT_GOOGLE_API_KEY` environment variable, never on the command
//! line (where other programs could read it). What went wrong goes to standard error, in
//! words for the person, and the program ends with a failure.

use cutcraft_borders::{
    DEPTH_PROMPT, KEY_VARIABLE, RATIOS, STYLES, ask, mime_of, picture_prompt, request,
};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let done = match args.as_slice() {
        ["styles"] => {
            for s in &STYLES {
                println!("{}\t{}", s.id, s.name);
            }
            Ok(())
        }
        ["picture", style, ratio, details, out] => ratio_ok(ratio).and_then(|()| {
            let body = request(&picture_prompt(style, details), None, ratio);
            key()
                .and_then(|k| ask(&k, &body))
                .and_then(|p| write(out, &p))
        }),
        ["depth", picture, ratio, out] => ratio_ok(ratio).and_then(|()| {
            let bytes = std::fs::read(picture)
                .map_err(|e| format!("could not read the picture {picture}: {e}"))?;
            let body = request(DEPTH_PROMPT, Some((mime_of(&bytes), &bytes)), ratio);
            key()
                .and_then(|k| ask(&k, &body))
                .and_then(|p| write(out, &p))
        }),
        _ => {
            eprintln!(
                "usage: cutcraft-borders picture STYLE RATIO DETAILS OUT\n       \
                 cutcraft-borders depth PICTURE RATIO OUT\n       cutcraft-borders styles"
            );
            return ExitCode::from(2);
        }
    };
    match done {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// The person's key, from the environment.
fn key() -> Result<String, String> {
    std::env::var(KEY_VARIABLE)
        .ok()
        .map(|k| k.trim().to_owned())
        .filter(|k| !k.is_empty())
        .ok_or_else(|| "No Google key was given: add yours in the form's Google key box".into())
}

fn ratio_ok(ratio: &str) -> Result<(), String> {
    if RATIOS.contains(&ratio) {
        Ok(())
    } else {
        Err(format!(
            "the tile shape must be one of {}",
            RATIOS.join(", ")
        ))
    }
}

fn write(out: &str, picture: &[u8]) -> Result<(), String> {
    std::fs::write(out, picture).map_err(|e| format!("could not write {out}: {e}"))
}
