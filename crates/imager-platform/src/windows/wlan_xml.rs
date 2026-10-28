//! The saved key in a WLAN profile (`WlanGetProfile` with the plaintext
//! key flag, upstream `winwlancredentials.cpp`). Pure, tested on every
//! host.

/// The `<keyMaterial>` text, XML entities decoded.
pub fn key_material(profile_xml: &str) -> Option<String> {
    let start = profile_xml.find("<keyMaterial>")? + "<keyMaterial>".len();
    let len = profile_xml[start..].find("</keyMaterial>")?;
    let key = unescape(profile_xml[start..start + len].trim());
    (!key.is_empty()).then_some(key)
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_plaintext_key() {
        let xml = "<WLANProfile><name>Home</name><MSM><security><sharedKey>\
                   <keyType>passPhrase</keyType><protected>false</protected>\
                   <keyMaterial>p&amp;ss&lt;word&gt;</keyMaterial></sharedKey>\
                   </security></MSM></WLANProfile>";
        assert_eq!(key_material(xml).as_deref(), Some("p&ss<word>"));
        assert_eq!(
            key_material("<WLANProfile><name>Open</name></WLANProfile>"),
            None
        );
        assert_eq!(key_material("<keyMaterial></keyMaterial>"), None);
        assert_eq!(key_material("<keyMaterial>unterminated"), None);
    }
}
