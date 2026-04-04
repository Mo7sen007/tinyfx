//! # icons
//!
//! Font Awesome icon inlining utilities.

use std::sync::OnceLock;

use regex::Regex;

pub fn inline_fontawesome_icons(html: &str) -> String {
    i_tag_re()
        .replace_all(html, |caps: &regex::Captures| {
            let whole = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
            let attrs = caps.name("attrs").map(|m| m.as_str()).unwrap_or_default();
            let Some(class_value) = extract_class_attr(attrs) else {
                return whole.to_string();
            };

            let class_parts: Vec<&str> = class_value.split_whitespace().collect();
            let Some((class_token, svg)) = bundled_icons()
                .iter()
                .find(|(token, _)| class_token_matches(&class_parts, token))
            else {
                return whole.to_string();
            };

            let mut data_attrs = String::new();
            for cap in data_attr_re().captures_iter(attrs) {
                let Some(full) = cap.get(0) else { continue };
                data_attrs.push(' ');
                data_attrs.push_str(full.as_str().trim());
            }

            format!(
                "<svg viewBox=\"0 0 512 512\" fill=\"currentColor\" aria-hidden=\"true\" focusable=\"false\" class=\"tinyfx-fa tinyfx-fa-{}\"{}>{}</svg>",
                class_token.replace(' ', "-"),
                data_attrs,
                svg
            )
        })
        .to_string()
}

pub fn warn_fontawesome_cdn_usage(html: &str) -> Vec<String> {
    if !cdn_re().is_match(html) {
        return Vec::new();
    }

    vec![
        "Font Awesome CDN stylesheet detected; TinyFX inlines supported icons and you can remove the CDN link for better LCP"
            .to_string(),
    ]
}

fn bundled_icons() -> &'static [(&'static str, &'static str)] {
    &[
        ("fa-solid fa-bolt", "<path d=\"M296 160h-94l18-100c3-16-9-28-26-28h-46c-13 0-24 9-27 22l-41 195c-4 18 10 35 29 35h83l-19 116c-3 16 16 26 29 15l212-186c17-15 7-43-18-43z\"/>") ,
        ("fa-solid fa-check", "<path d=\"M173 439L7 273l45-45 121 121L460 62l45 45z\"/>") ,
        ("fa-solid fa-xmark", "<path d=\"M405 109L299 215 193 109l-45 45 106 106-106 106 45 45 106-106 106 106 45-45-106-106 106-106z\"/>") ,
        ("fa-solid fa-plus", "<path d=\"M288 64v160h160v64H288v160h-64V288H64v-64h160V64z\"/>") ,
        ("fa-solid fa-minus", "<path d=\"M64 224h384v64H64z\"/>") ,
        ("fa-solid fa-trash", "<path d=\"M160 64h192l16 32h80v32H64V96h80zM96 160h320l-21 288H117z\"/>") ,
        ("fa-solid fa-pen", "<path d=\"M96 352l192-192 64 64-192 192H96zM416 160l-32 32-64-64 32-32c9-9 23-9 32 0l32 32c9 9 9 23 0 32z\"/>") ,
        ("fa-solid fa-floppy-disk", "<path d=\"M64 32h320l64 64v384H64zM128 96v96h192V96zM128 288v96h256v-96z\"/>") ,
        ("fa-solid fa-copy", "<path d=\"M160 32h288v352H160zM64 128h288v352H64z\"/>") ,
        ("fa-solid fa-download", "<path d=\"M256 64v224l80-80 45 45-157 157L67 253l45-45 80 80V64zM64 416h384v64H64z\"/>") ,
        ("fa-solid fa-upload", "<path d=\"M256 448V224l80 80 45-45-157-157L67 259l45 45 80-80v224zM64 32h384v64H64z\"/>") ,
        ("fa-solid fa-arrow-left", "<path d=\"M256 64L64 256l192 192 45-45-115-115h262v-64H186l115-115z\"/>") ,
        ("fa-solid fa-arrow-right", "<path d=\"M256 64l-45 45 115 115H64v64h262l-115 115 45 45 192-192z\"/>") ,
        ("fa-solid fa-arrow-up", "<path d=\"M64 256l45 45 115-115v262h64V186l115 115 45-45L256 64z\"/>") ,
        ("fa-solid fa-arrow-down", "<path d=\"M64 256l192 192 192-192-45-45-115 115V64h-64v267L109 211z\"/>") ,
        ("fa-solid fa-chevron-left", "<path d=\"M320 64L128 256l192 192 45-45-147-147 147-147z\"/>") ,
        ("fa-solid fa-chevron-right", "<path d=\"M192 64l-45 45 147 147-147 147 45 45 192-192z\"/>") ,
        ("fa-solid fa-chevron-up", "<path d=\"M64 320l45 45 147-147 147 147 45-45-192-192z\"/>") ,
        ("fa-solid fa-chevron-down", "<path d=\"M64 192l192 192 192-192-45-45-147 147-147-147z\"/>") ,
        ("fa-solid fa-bars", "<path d=\"M64 96h384v48H64zM64 232h384v48H64zM64 368h384v48H64z\"/>") ,
        ("fa-solid fa-house", "<path d=\"M256 64L48 224h48v224h128V320h64v128h128V224h48z\"/>") ,
        ("fa-solid fa-magnifying-glass", "<path d=\"M224 64a160 160 0 11113 273l111 111-45 45-111-111A160 160 0 01224 64z\"/>") ,
        ("fa-solid fa-bell", "<path d=\"M256 32c70 0 128 58 128 128v80l32 64H96l32-64v-80c0-70 58-128 128-128zM192 352a64 64 0 00128 0z\"/>") ,
        ("fa-solid fa-gear", "<path d=\"M256 160a96 96 0 100 192 96 96 0 000-192zm-182 96l36-21-2-43 40-16 24-35 41 11 34-23 34 23 41-11 24 35 40 16-2 43 36 21-15 40-36 21 2 43-40 16-24 35-41-11-34 23-34-23-41 11-24-35-40-16 2-43-36-21z\"/>") ,
        ("fa-solid fa-spinner", "<path d=\"M256 64a192 192 0 10192 192h-64a128 128 0 11-128-128z\"/>") ,
        ("fa-solid fa-ellipsis", "<path d=\"M96 224a32 32 0 110 64 32 32 0 010-64zm160 0a32 32 0 110 64 32 32 0 010-64zm160 0a32 32 0 110 64 32 32 0 010-64z\"/>") ,
        ("fa-solid fa-circle-check", "<path d=\"M256 32a224 224 0 110 448 224 224 0 010-448zm-35 289l141-141-45-45-96 96-45-45-45 45z\"/>") ,
        ("fa-solid fa-circle-exclamation", "<path d=\"M256 32a224 224 0 110 448 224 224 0 010-448zm-24 112v160h48V144zm0 192v48h48v-48z\"/>") ,
        ("fa-solid fa-circle-info", "<path d=\"M256 32a224 224 0 110 448 224 224 0 010-448zm-24 176v144h48V208zm0-80v48h48v-48z\"/>") ,
        ("fa-solid fa-circle-question", "<path d=\"M256 32a224 224 0 110 448 224 224 0 010-448zm0 320a32 32 0 100 64 32 32 0 000-64zm-48-176h48c26 0 48 22 48 48 0 20-10 31-25 43-13 10-23 18-23 37v16h-48v-24c0-30 15-47 33-61 8-6 15-10 15-19 0-8-7-16-16-16h-32z\"/>") ,
        ("fa-solid fa-image", "<path d=\"M64 96h384v320H64zm64 64a32 32 0 100 64 32 32 0 000-64zm32 192l64-64 64 64 96-96 32 32v64H160z\"/>") ,
        ("fa-solid fa-video", "<path d=\"M64 128h256v256H64zM352 192l96-64v256l-96-64z\"/>") ,
        ("fa-solid fa-play", "<path d=\"M160 96l224 160-224 160z\"/>") ,
        ("fa-solid fa-pause", "<path d=\"M128 96h96v320h-96zM288 96h96v320h-96z\"/>") ,
        ("fa-solid fa-stop", "<path d=\"M96 96h320v320H96z\"/>") ,
        ("fa-solid fa-volume-high", "<path d=\"M64 192h96l96-96v320l-96-96H64zM320 176a80 80 0 010 160v-48a32 32 0 000-64z\"/>") ,
        ("fa-solid fa-volume-xmark", "<path d=\"M64 192h96l96-96v320l-96-96H64zm256 32l32 32 32-32 32 32-32 32 32 32-32 32-32-32-32 32-32-32 32-32-32-32z\"/>") ,
        ("fa-solid fa-envelope", "<path d=\"M64 128h384v256H64zm32 32l160 112 160-112z\"/>") ,
        ("fa-solid fa-phone", "<path d=\"M152 64l64 64-32 64c31 60 76 105 136 136l64-32 64 64-64 96c-192 0-352-160-352-352z\"/>") ,
        ("fa-solid fa-comment", "<path d=\"M64 96h384v224H192l-96 96v-96H64z\"/>") ,
        ("fa-solid fa-comments", "<path d=\"M64 96h256v160H128l-64 64v-64H64zm128 96h256v160h-64v64l-64-64H192z\"/>") ,
        ("fa-solid fa-share", "<path d=\"M352 96a64 64 0 110 128 64 64 0 010-128zm-192 96l192-32v64zM352 288a64 64 0 110 128 64 64 0 010-128z\"/>") ,
        ("fa-solid fa-user", "<path d=\"M256 64a96 96 0 110 192 96 96 0 010-192zM64 416c0-88 104-128 192-128s192 40 192 128z\"/>") ,
        ("fa-solid fa-users", "<path d=\"M192 96a80 80 0 110 160 80 80 0 010-160zm160 16a64 64 0 110 128 64 64 0 010-128zM32 416c0-80 96-112 160-112s160 32 160 112z\"/>") ,
        ("fa-solid fa-lock", "<path d=\"M128 224v-64a128 128 0 11256 0v64h32v224H96V224zm64 0h128v-64a64 64 0 10-128 0z\"/>") ,
        ("fa-solid fa-unlock", "<path d=\"M384 224h32v224H96V224h224v-64a64 64 0 10-128 0h-64a128 128 0 11256 0z\"/>") ,
        ("fa-solid fa-key", "<path d=\"M192 256a96 96 0 11191 16L512 272v80h-64v64h-64v-64h-64l-81-81a96 96 0 01-47-15z\"/>") ,
        ("fa-solid fa-file", "<path d=\"M128 32h192l96 96v352H128zM320 32v128h128\"/>") ,
        ("fa-solid fa-folder", "<path d=\"M64 128h160l32 32h192v256H64z\"/>") ,
        ("fa-solid fa-database", "<path d=\"M256 64c106 0 192 29 192 64v256c0 35-86 64-192 64S64 419 64 384V128c0-35 86-64 192-64z\"/>") ,
        ("fa-solid fa-table", "<path d=\"M64 96h384v320H64zm0 96h384M192 96v320M320 96v320\"/>") ,
        ("fa-solid fa-star", "<path d=\"M256 64l58 118 130 19-94 92 22 131-116-61-116 61 22-131-94-92 130-19z\"/>") ,
        ("fa-solid fa-heart", "<path d=\"M256 448l-32-29C104 312 32 248 32 160 32 96 80 48 144 48c40 0 78 18 112 52 34-34 72-52 112-52 64 0 112 48 112 112 0 88-72 152-192 259z\"/>") ,
        ("fa-solid fa-bookmark", "<path d=\"M128 64h256v384l-128-64-128 64z\"/>") ,
        ("fa-solid fa-flag", "<path d=\"M96 64h32v384H96zM128 96h256l-32 64 32 64H128z\"/>") ,
        ("fa-solid fa-tag", "<path d=\"M64 224L224 64h160v160L224 384z\"/>") ,
        ("fa-solid fa-link", "<path d=\"M181 331l-45 45a96 96 0 01-136-136l77-77a96 96 0 01136 0l45 45-45 45-45-45a32 32 0 00-45 0l-77 77a32 32 0 0045 45l45-45zm150-150l45-45a96 96 0 01136 136l-77 77a96 96 0 01-136 0l-45-45 45-45 45 45a32 32 0 0045 0l77-77a32 32 0 00-45-45l-45 45z\"/>") ,
        ("fa-solid fa-up-right-from-square", "<path d=\"M288 64h160v160h-64v-51L197 360l-45-45 187-187h-51zM96 96h128v64H160v256h256v-64h64v128H96z\"/>") ,
        ("fa-solid fa-fire", "<path d=\"M290 32c18 80-46 93-34 160 8 46 58 62 58 128 0 65-52 116-116 116S82 385 82 320c0-93 70-140 128-192 26-23 51-49 80-96z\"/>") ,
        ("fa-solid fa-sun", "<path d=\"M256 144a112 112 0 110 224 112 112 0 010-224zm0-96h32v64h-32zm0 352h32v64h-32zM48 240h64v32H48zm352 0h64v32h-64zM97 97l45 45-22 22-45-45zm272 272l45 45-22 22-45-45zM97 415l45-45 22 22-45 45zm272-272l45-45 22 22-45 45z\"/>") ,
        ("fa-solid fa-moon", "<path d=\"M352 352c-106 0-192-86-192-192 0-44 15-84 40-116C99 56 32 146 32 256c0 124 100 224 224 224 110 0 200-67 212-168-32 25-72 40-116 40z\"/>") ,
    ]
}

fn class_token_matches(class_parts: &[&str], class_token: &str) -> bool {
    class_token
        .split_whitespace()
        .all(|required| class_parts.iter().any(|part| *part == required))
}

fn extract_class_attr(attrs: &str) -> Option<String> {
    let caps = class_attr_re().captures(attrs)?;
    if let Some(v) = caps.get(1) {
        return Some(v.as_str().to_string());
    }
    if let Some(v) = caps.get(2) {
        return Some(v.as_str().to_string());
    }
    None
}

fn i_tag_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"(?is)<i(?P<attrs>[^>]*)>\s*</i>"#).unwrap())
}

fn class_attr_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"class\s*=\s*\"([^\"]*)\"|class\s*=\s*'([^']*)'"#).unwrap())
}

fn data_attr_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"\bdata-[a-zA-Z0-9_-]+\s*=\s*\"[^\"]*\"|\bdata-[a-zA-Z0-9_-]+\s*=\s*'[^']*'"#)
            .unwrap()
    })
}

fn cdn_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r#"(?i)https?://[^\"']*fontawesome[^\"']*|https?://[^\"']*cdnjs[^\"']*font-awesome[^\"']*"#).unwrap()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inlines_supported_icon() {
        let html = r#"<div><i class="fa-solid fa-bolt"></i></div>"#;
        let out = inline_fontawesome_icons(html);
        assert!(out.contains("<svg"));
        assert!(!out.contains("<i class=\"fa-solid fa-bolt\""));
    }

    #[test]
    fn preserves_data_attrs_on_icon() {
        let html = r#"<i class="fa-solid fa-check" data-id="ok" data-role="status"></i>"#;
        let out = inline_fontawesome_icons(html);
        assert!(out.contains("data-id=\"ok\""));
        assert!(out.contains("data-role=\"status\""));
    }

    #[test]
    fn warns_when_cdn_found() {
        let html = r#"<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.0/css/all.min.css">"#;
        let warnings = warn_fontawesome_cdn_usage(html);
        assert_eq!(warnings.len(), 1);
    }
}
