//! Localisation data bundled with upstream (`timezones.txt`,
//! `countries.txt`, `keymap-layouts.txt`, `capital-cities.txt`) and the
//! lookups the Localisation step uses.

use serde::Serialize;

const TIMEZONES: &str = include_str!("../data/timezones.txt");
const COUNTRIES: &str = include_str!("../data/countries.txt");
const KEYMAPS: &str = include_str!("../data/keymap-layouts.txt");
const CAPITALS: &str = include_str!("../data/capital-cities.txt");

/// IANA time zones. Like upstream (`split('\n')` without skipping empty
/// parts) the list ends with one empty entry.
pub fn timezones() -> Vec<String> {
    TIMEZONES.split('\n').map(|s| s.trim().to_owned()).collect()
}

/// ISO 3166 Wi-Fi regulatory country codes.
pub fn countries() -> Vec<String> {
    non_empty_lines(COUNTRIES)
}

/// Console keymap layout codes.
pub fn keymap_layouts() -> Vec<String> {
    non_empty_lines(KEYMAPS)
}

/// "City (Country)" display names in file order.
pub fn capital_cities() -> Vec<String> {
    capital_rows()
        .filter(|p| p.len() >= 2)
        .map(|p| format!("{} ({})", p[0], p[1]))
        .collect()
}

/// Locale defaults of one capital (`getLocaleDataForCapital`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapitalLocale {
    pub city_name: String,
    pub country_name: String,
    pub country_code: String,
    pub timezone: String,
    pub language: String,
    pub keyboard: String,
}

/// Looks up a capital by "City (Country)" or bare city name.
pub fn locale_for_capital(display_name: &str) -> Option<CapitalLocale> {
    let city = match display_name.find(" (") {
        Some(index) if index > 0 => &display_name[..index],
        _ => display_name,
    };
    capital_rows()
        .find(|p| p.len() >= 6 && p[0] == city)
        .map(|p| CapitalLocale {
            city_name: p[0].to_owned(),
            country_name: p[1].to_owned(),
            country_code: p[2].to_owned(),
            timezone: p[3].to_owned(),
            language: p[4].to_owned(),
            keyboard: p[5].to_owned(),
        })
}

fn capital_rows() -> impl Iterator<Item = Vec<&'static str>> {
    CAPITALS
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.split('|').collect())
}

fn non_empty_lines(text: &str) -> Vec<String> {
    text.split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timezone_list_keeps_upstream_trailing_empty_entry() {
        let zones = timezones();
        assert_eq!(zones.len(), 433);
        assert_eq!(zones[0], "Africa/Abidjan");
        assert_eq!(zones.last().map(String::as_str), Some(""));
    }

    #[test]
    fn country_and_keymap_lists_are_complete() {
        assert_eq!(countries().len(), 173);
        assert_eq!(countries()[0], "AD");
        assert_eq!(keymap_layouts().len(), 98);
        assert!(keymap_layouts().contains(&"gb".to_owned()));
    }

    #[test]
    fn capitals_are_city_country_pairs() {
        let cities = capital_cities();
        assert_eq!(cities.len(), 246);
        assert_eq!(cities[0], "Abu Dhabi (United Arab Emirates)");
        assert!(cities.contains(&"London (United Kingdom)".to_owned()));
    }

    #[test]
    fn looks_up_capital_locale() {
        let london = locale_for_capital("London (United Kingdom)").unwrap();
        assert_eq!(london.country_code, "GB");
        assert_eq!(london.timezone, "Europe/London");
        assert_eq!(london.keyboard, "gb");
        assert_eq!(locale_for_capital("Abuja").unwrap().country_code, "NG");
        assert!(locale_for_capital("Atlantis (Nowhere)").is_none());
        assert!(locale_for_capital(" (x)").is_none());
    }
}
