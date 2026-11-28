//! What the Borders add-on asks Google's image model, and how it reads the answer. Cut
//! Craft runs the program (`main.rs`) behind its Generate Border form: first for a picture of
//! one repeat of a moulding in the chosen style, then for that very picture's depth map, so
//! the depth and the picture are the same design. The person's own Google key comes in the
//! `CUTCRAFT_GOOGLE_API_KEY` environment variable; it is never written anywhere.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::time::Duration;

/// The environment variable the key comes in.
pub const KEY_VARIABLE: &str = "CUTCRAFT_GOOGLE_API_KEY";
/// The environment variable that may name another image model (for when Google renames
/// theirs before this add-on is updated).
pub const MODEL_VARIABLE: &str = "CUTCRAFT_GOOGLE_IMAGE_MODEL";
/// The image model asked, unless [`MODEL_VARIABLE`] names another.
pub const MODEL: &str = "gemini-3.1-flash-image";

/// One of the styles, and how it is described to the model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    /// Its id, as PowerWhittle's form sends it.
    pub id: &'static str,
    pub name: &'static str,
    /// The moulding, as the model is told it.
    pub description: &'static str,
}

/// The styles, in the form's order.
pub const STYLES: [Style; 9] = [
    Style {
        id: "classical_baroque",
        name: "Classical Baroque",
        description: "antique gilded gold baroque moulding with deep carved acanthus leaf \
            scrolls, a central shell cartouche, flowing C-scrolls and a fine beaded lower rail",
    },
    Style {
        id: "modern_minimalist",
        name: "Modern Minimalist",
        description: "sleek matte black modern moulding with a clean stepped profile, crisp \
            straight parallel reeds and no ornament",
    },
    Style {
        id: "art_deco",
        name: "Art Deco",
        description: "brushed brass art deco moulding with a stepped geometric ziggurat, \
            radiating sunburst fan lines and crisp chevron bands between straight rails",
    },
    Style {
        id: "rustic_vintage",
        name: "Rustic Vintage",
        description: "weathered reclaimed barn wood moulding with deep open grain, knots, worn \
            rounded edges, adze marks and hand-hewn distressing",
    },
    Style {
        id: "victorian_carved",
        name: "Victorian Carved",
        description: "dark stained walnut hardwood moulding with intricate Victorian relief \
            carving: a central oval egg-and-dart / bead-and-reel band, a lower floral rosette \
            frieze and a fine pearl bead border",
    },
    Style {
        id: "contemporary_metallic",
        name: "Contemporary Metallic",
        description: "brushed champagne silver satin moulding with a wide central band of fine \
            diagonal diamond crosshatch knurling framed by precision-milled parallel grooved \
            rails",
    },
    Style {
        id: "rococo",
        name: "Rococo",
        description: "light champagne-gold and cream gilded rococo moulding with delicate \
            symmetrical scallop shells, intertwined C-scrolls, botanical acanthus curls and fine \
            beaded upper and lower borders",
    },
    Style {
        id: "mid_century_modern",
        name: "Mid-Century Modern",
        description: "warm oiled American walnut moulding with a smooth convex curved bullnose \
            profile inlaid with two crisp thin parallel brushed brass strips near the top and \
            bottom edges",
    },
    Style {
        id: "gothic_pewter",
        name: "Gothic Pewter",
        description: "aged dark pewter and forged iron trim with repeating pointed gothic lancet \
            arches, carved quatrefoil and trefoil tracery and heavy rolled horizontal edge \
            mouldings",
    },
];

/// The style with id `id`; any other id means "custom", described only by the details.
pub fn style(id: &str) -> Option<&'static Style> {
    STYLES.iter().find(|s| s.id == id)
}

/// What every request says about how one repeat must be drawn so its ends meet.
const TILING: &str = "CRITICAL SEAMLESS TILING REQUIREMENT: show ONLY ONE horizontally \
    repeating unit of this border. The pattern must be strictly periodic so the exact left \
    edge of the image joins the exact right edge with no seam, no perspective, no vignette \
    and no change of lighting from left to right. Straight-on, flat, orthographic front view.";

/// What every request says about framing.
const FRAMING: &str = "A full-bleed horizontal moulding strip spanning 100% from the left \
    edge to the right edge and filling the frame top to bottom, its top and bottom rails \
    straight and parallel to the image's top and bottom edges.";

/// What the model is asked for a picture of the style `style_id` with the extra `details`.
pub fn picture_prompt(style_id: &str, details: &str) -> String {
    let details = details.trim();
    let subject = match style(style_id) {
        Some(s) if details.is_empty() => s.description.to_owned(),
        Some(s) => format!("{}, incorporating {details}", s.description),
        None if details.is_empty() => {
            "ornate carved architectural picture-frame moulding with classical relief".to_owned()
        }
        None => format!("{details} architectural decorative picture-frame moulding border"),
    };
    format!(
        "Photorealistic macro studio product photograph of a single horizontally tileable \
         architectural picture-frame moulding section: {subject}. Even, diffused \
         orthographic studio lighting square to the surface so the left and right edges have \
         identical brightness and tone. Crisp tactile relief detail, realistic material \
         finish, no perspective skew. {FRAMING} {TILING}"
    )
}

/// What the model is asked, with the picture, for the picture's depth map.
pub const DEPTH_PROMPT: &str = "Convert this exact picture of a carved moulding into its \
    bas-relief depth map (height map) for a CNC router. Keep the very same design, framing, \
    size and position of every element. Pure linear greyscale: black (#000000) is the \
    deepest, lowest background; white (#FFFFFF) is the highest raised surface. NO lighting, \
    NO shadows, NO ambient occlusion, NO highlights, NO colour, NO wood grain or texture \
    noise: only the surface's height. Smooth, rounded, convex carved forms. The left and \
    right edges must still join seamlessly.";

/// The aspect ratios the model is asked for, as the form offers them.
pub const RATIOS: [&str; 3] = ["21:9", "16:9", "4:3"];

/// The request body: `prompt`, with `picture` (its MIME type and bytes) when one is sent,
/// for an image `ratio` wide to high.
pub fn request(prompt: &str, picture: Option<(&str, &[u8])>, ratio: &str) -> Value {
    let mut parts = vec![json!({ "text": prompt })];
    if let Some((mime, bytes)) = picture {
        parts.push(json!({ "inlineData": { "mimeType": mime, "data": STANDARD.encode(bytes) } }));
    }
    json!({
        "contents": [{ "role": "user", "parts": parts }],
        "generationConfig": {
            "responseModalities": ["IMAGE"],
            "imageConfig": { "aspectRatio": ratio }
        }
    })
}

/// The picture in the model's answer `answer`: its bytes, or what to tell the person.
pub fn picture_in(answer: &Value) -> Result<Vec<u8>, String> {
    if let Some(message) = answer.pointer("/error/message").and_then(Value::as_str) {
        return Err(format!("Google said: {message}"));
    }
    let parts = answer
        .pointer("/candidates/0/content/parts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let data = parts
        .iter()
        .find_map(|p| p.pointer("/inlineData/data").and_then(Value::as_str));
    match data {
        Some(d) => STANDARD
            .decode(d)
            .map_err(|e| format!("Google's picture could not be read ({e})")),
        None => {
            let why = answer
                .pointer("/promptFeedback/blockReason")
                .or_else(|| answer.pointer("/candidates/0/finishReason"))
                .and_then(Value::as_str);
            Err(match why {
                Some(why) => format!(
                    "Google made no picture ({why}). Change the extra details and try again"
                ),
                None => "Google made no picture. Try again, or change the extra details".into(),
            })
        }
    }
}

/// What to tell the person when Google answers with `code`.
pub fn refused(code: u16, said: Option<&str>) -> String {
    let said = said
        .map(|s| format!(" Google said: {s}"))
        .unwrap_or_default();
    match code {
        400 => format!("Google could not use the request.{said}"),
        401 | 403 => format!(
            "Google did not accept the key. Check it in the form's Google key box, and that \
             image generation is turned on for it.{said}"
        ),
        404 => format!(
            "Google has no image model called \"{}\". Update the Borders add-on (Gadgets > \
             Add-ons).{said}",
            model()
        ),
        429 => format!(
            "Google says this key has made too many pictures for now, or has no credit left. \
             Wait a minute and try again.{said}"
        ),
        _ => format!("Google answered with error {code}.{said} Try again in a minute."),
    }
}

/// The image model to ask.
pub fn model() -> String {
    std::env::var(MODEL_VARIABLE)
        .ok()
        .filter(|m| !m.trim().is_empty())
        .unwrap_or_else(|| MODEL.to_owned())
}

/// Asks the model `body`, with the key `key` (sent in a header, never in the address, so it
/// stays out of any log of addresses), trying again a few times when Google is busy.
pub fn ask(key: &str, body: &Value) -> Result<Vec<u8>, String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model()
    );
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_global(Some(Duration::from_secs(240)))
        .http_status_as_error(false)
        .user_agent(concat!("CutCraft-Borders/", env!("CARGO_PKG_VERSION")))
        .tls_config(
            ureq::tls::TlsConfig::builder()
                .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .into();
    let mut wait = Duration::from_secs(2);
    for attempt in 0..4 {
        let sent = agent
            .post(&url)
            .header("x-goog-api-key", key)
            .send_json(body);
        let mut response = match sent {
            Ok(r) => r,
            Err(e) if attempt < 3 => {
                eprintln!("could not reach Google ({e}); trying again");
                std::thread::sleep(wait);
                wait *= 2;
                continue;
            }
            Err(e) => {
                return Err(format!(
                    "Could not reach Google. Check this computer is online. ({e})"
                ));
            }
        };
        let code = response.status().as_u16();
        let answer: Value = response
            .body_mut()
            .with_config()
            .limit(64 * 1024 * 1024)
            .read_json()
            .unwrap_or(Value::Null);
        if code == 200 {
            return picture_in(&answer);
        }
        if (code == 429 || code >= 500) && attempt < 3 {
            eprintln!("Google is busy ({code}); trying again");
            std::thread::sleep(wait);
            wait *= 2;
            continue;
        }
        return Err(refused(
            code,
            answer.pointer("/error/message").and_then(Value::as_str),
        ));
    }
    Err("Google is busy. Try again in a minute".into())
}

/// The MIME type of the picture `bytes`, from its first bytes.
pub fn mime_of(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "image/png"
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        "image/jpeg"
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        "image/webp"
    } else {
        "application/octet-stream"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_style_has_its_own_description_and_details_are_added() {
        for s in &STYLES {
            let p = picture_prompt(s.id, "");
            assert!(
                p.contains(s.description) && p.contains("SEAMLESS"),
                "{}",
                s.id
            );
        }
        let p = picture_prompt("rococo", " laurel wreath ");
        assert!(p.contains("incorporating laurel wreath."), "{p}");
        let custom = picture_prompt("custom", "carved jade dragon");
        assert!(
            custom.contains("carved jade dragon architectural"),
            "{custom}"
        );
    }

    #[test]
    fn the_request_carries_the_picture_and_the_ratio() {
        let r = request("make it", Some(("image/png", b"abc")), "16:9");
        assert_eq!(r["contents"][0]["parts"][1]["inlineData"]["data"], "YWJj");
        assert_eq!(r["generationConfig"]["imageConfig"]["aspectRatio"], "16:9");
        assert_eq!(
            request("x", None, "4:3")["contents"][0]["parts"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn the_answer_gives_its_picture_or_says_why_there_is_none() {
        let ok = json!({"candidates":[{"content":{"parts":[{"text":"here"},{"inlineData":{"mimeType":"image/png","data":"YWJj"}}]}}]});
        assert_eq!(picture_in(&ok).unwrap(), b"abc");
        let blocked = json!({"promptFeedback":{"blockReason":"SAFETY"}});
        assert!(picture_in(&blocked).unwrap_err().contains("SAFETY"));
        let error = json!({"error":{"message":"API key not valid"}});
        assert!(
            picture_in(&error)
                .unwrap_err()
                .contains("API key not valid")
        );
    }

    #[test]
    fn refusals_say_what_to_do() {
        assert!(refused(403, None).contains("key"));
        assert!(refused(429, None).contains("Wait"));
        assert!(refused(404, None).contains("Update the Borders add-on"));
    }

    #[test]
    fn pictures_are_told_apart_by_their_first_bytes() {
        assert_eq!(mime_of(&[0x89, b'P', b'N', b'G', 0]), "image/png");
        assert_eq!(mime_of(&[0xFF, 0xD8, 0xFF]), "image/jpeg");
    }
}
