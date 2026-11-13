//! Guest phone numbers: normalisation to E.164 and the country calling codes
//! the booking form and the admin panel offer.
//!
//! Gateways all want E.164 (`+33612345678`), while guests type what they type:
//! `06 12 34 56 78`, `0033612345678`, `+33 6 12 34 56 78`. Normalisation
//! happens once, server-side, at booking time; everything downstream assumes
//! `bookings.guest_phone` is already E.164.
//!
//! This is deliberately not libphonenumber. It handles the trunk-prefix and
//! international-prefix cases that cover ordinary input, and leaves genuine
//! validity to the gateway, which is the only real authority on whether a
//! number can receive a message.

/// Used when nothing is configured yet, so the booking form can render before
/// an admin has been through the panel.
pub const DEFAULT_COUNTRY_CODE: &str = "+33";

/// Country calling codes offered in the admin panel and used to normalise
/// locally formatted guest numbers: (code, ISO 3166-1 alpha-2 regions, label).
///
/// ilia fork: the regions (space-separated, several when a code is shared)
/// let the admin page rename each option in the page language with
/// `Intl.DisplayNames`; the English label is the no-JS fallback.
pub const COUNTRY_CODES: &[(&str, &str, &str)] = &[
    ("+1", "US CA", "United States / Canada (+1)"),
    ("+20", "EG", "Egypt (+20)"),
    ("+27", "ZA", "South Africa (+27)"),
    ("+30", "GR", "Greece (+30)"),
    ("+31", "NL", "Netherlands (+31)"),
    ("+32", "BE", "Belgium (+32)"),
    ("+33", "FR", "France (+33)"),
    ("+34", "ES", "Spain (+34)"),
    ("+36", "HU", "Hungary (+36)"),
    ("+39", "IT", "Italy (+39)"),
    ("+40", "RO", "Romania (+40)"),
    ("+41", "CH", "Switzerland (+41)"),
    ("+43", "AT", "Austria (+43)"),
    ("+44", "GB", "United Kingdom (+44)"),
    ("+45", "DK", "Denmark (+45)"),
    ("+46", "SE", "Sweden (+46)"),
    ("+47", "NO", "Norway (+47)"),
    ("+48", "PL", "Poland (+48)"),
    ("+49", "DE", "Germany (+49)"),
    ("+52", "MX", "Mexico (+52)"),
    ("+53", "CU", "Cuba (+53)"),
    ("+54", "AR", "Argentina (+54)"),
    ("+55", "BR", "Brazil (+55)"),
    ("+56", "CL", "Chile (+56)"),
    ("+57", "CO", "Colombia (+57)"),
    ("+58", "VE", "Venezuela (+58)"),
    ("+60", "MY", "Malaysia (+60)"),
    ("+61", "AU", "Australia (+61)"),
    ("+62", "ID", "Indonesia (+62)"),
    ("+63", "PH", "Philippines (+63)"),
    ("+64", "NZ", "New Zealand (+64)"),
    ("+65", "SG", "Singapore (+65)"),
    ("+66", "TH", "Thailand (+66)"),
    ("+7", "RU KZ", "Russia / Kazakhstan (+7)"),
    ("+81", "JP", "Japan (+81)"),
    ("+82", "KR", "South Korea (+82)"),
    ("+84", "VN", "Vietnam (+84)"),
    ("+86", "CN", "China (+86)"),
    ("+90", "TR", "Turkey (+90)"),
    ("+91", "IN", "India (+91)"),
    ("+92", "PK", "Pakistan (+92)"),
    ("+93", "AF", "Afghanistan (+93)"),
    ("+94", "LK", "Sri Lanka (+94)"),
    ("+95", "MM", "Myanmar (+95)"),
    ("+98", "IR", "Iran (+98)"),
    ("+212", "MA", "Morocco (+212)"),
    ("+213", "DZ", "Algeria (+213)"),
    ("+216", "TN", "Tunisia (+216)"),
    ("+218", "LY", "Libya (+218)"),
    ("+220", "GM", "Gambia (+220)"),
    ("+221", "SN", "Senegal (+221)"),
    ("+222", "MR", "Mauritania (+222)"),
    ("+223", "ML", "Mali (+223)"),
    ("+224", "GN", "Guinea (+224)"),
    ("+225", "CI", "Côte d'Ivoire (+225)"),
    ("+226", "BF", "Burkina Faso (+226)"),
    ("+227", "NE", "Niger (+227)"),
    ("+228", "TG", "Togo (+228)"),
    ("+229", "BJ", "Benin (+229)"),
    ("+230", "MU", "Mauritius (+230)"),
    ("+231", "LR", "Liberia (+231)"),
    ("+232", "SL", "Sierra Leone (+232)"),
    ("+233", "GH", "Ghana (+233)"),
    ("+234", "NG", "Nigeria (+234)"),
    ("+235", "TD", "Chad (+235)"),
    ("+236", "CF", "Central African Republic (+236)"),
    ("+237", "CM", "Cameroon (+237)"),
    ("+238", "CV", "Cape Verde (+238)"),
    ("+239", "ST", "São Tomé and Príncipe (+239)"),
    ("+240", "GQ", "Equatorial Guinea (+240)"),
    ("+241", "GA", "Gabon (+241)"),
    ("+242", "CG", "Republic of the Congo (+242)"),
    ("+243", "CD", "DR Congo (+243)"),
    ("+244", "AO", "Angola (+244)"),
    ("+245", "GW", "Guinea-Bissau (+245)"),
    ("+248", "SC", "Seychelles (+248)"),
    ("+249", "SD", "Sudan (+249)"),
    ("+250", "RW", "Rwanda (+250)"),
    ("+251", "ET", "Ethiopia (+251)"),
    ("+252", "SO", "Somalia (+252)"),
    ("+253", "DJ", "Djibouti (+253)"),
    ("+254", "KE", "Kenya (+254)"),
    ("+255", "TZ", "Tanzania (+255)"),
    ("+256", "UG", "Uganda (+256)"),
    ("+257", "BI", "Burundi (+257)"),
    ("+258", "MZ", "Mozambique (+258)"),
    ("+260", "ZM", "Zambia (+260)"),
    ("+261", "MG", "Madagascar (+261)"),
    ("+262", "RE YT", "Réunion / Mayotte (+262)"),
    ("+263", "ZW", "Zimbabwe (+263)"),
    ("+264", "NA", "Namibia (+264)"),
    ("+265", "MW", "Malawi (+265)"),
    ("+266", "LS", "Lesotho (+266)"),
    ("+267", "BW", "Botswana (+267)"),
    ("+268", "SZ", "Eswatini (+268)"),
    ("+269", "KM", "Comoros (+269)"),
    ("+290", "SH", "Saint Helena (+290)"),
    ("+291", "ER", "Eritrea (+291)"),
    ("+297", "AW", "Aruba (+297)"),
    ("+298", "FO", "Faroe Islands (+298)"),
    ("+299", "GL", "Greenland (+299)"),
    ("+350", "GI", "Gibraltar (+350)"),
    ("+351", "PT", "Portugal (+351)"),
    ("+352", "LU", "Luxembourg (+352)"),
    ("+353", "IE", "Ireland (+353)"),
    ("+354", "IS", "Iceland (+354)"),
    ("+355", "AL", "Albania (+355)"),
    ("+356", "MT", "Malta (+356)"),
    ("+357", "CY", "Cyprus (+357)"),
    ("+358", "FI", "Finland (+358)"),
    ("+359", "BG", "Bulgaria (+359)"),
    ("+370", "LT", "Lithuania (+370)"),
    ("+371", "LV", "Latvia (+371)"),
    ("+372", "EE", "Estonia (+372)"),
    ("+373", "MD", "Moldova (+373)"),
    ("+374", "AM", "Armenia (+374)"),
    ("+375", "BY", "Belarus (+375)"),
    ("+376", "AD", "Andorra (+376)"),
    ("+377", "MC", "Monaco (+377)"),
    ("+378", "SM", "San Marino (+378)"),
    ("+380", "UA", "Ukraine (+380)"),
    ("+381", "RS", "Serbia (+381)"),
    ("+382", "ME", "Montenegro (+382)"),
    ("+383", "XK", "Kosovo (+383)"),
    ("+385", "HR", "Croatia (+385)"),
    ("+386", "SI", "Slovenia (+386)"),
    ("+387", "BA", "Bosnia and Herzegovina (+387)"),
    ("+389", "MK", "North Macedonia (+389)"),
    ("+420", "CZ", "Czech Republic (+420)"),
    ("+421", "SK", "Slovakia (+421)"),
    ("+423", "LI", "Liechtenstein (+423)"),
    ("+500", "FK", "Falkland Islands (+500)"),
    ("+501", "BZ", "Belize (+501)"),
    ("+502", "GT", "Guatemala (+502)"),
    ("+503", "SV", "El Salvador (+503)"),
    ("+504", "HN", "Honduras (+504)"),
    ("+505", "NI", "Nicaragua (+505)"),
    ("+506", "CR", "Costa Rica (+506)"),
    ("+507", "PA", "Panama (+507)"),
    ("+508", "PM", "Saint Pierre and Miquelon (+508)"),
    ("+509", "HT", "Haiti (+509)"),
    ("+590", "GP MF", "Guadeloupe / Saint Martin (+590)"),
    ("+591", "BO", "Bolivia (+591)"),
    ("+592", "GY", "Guyana (+592)"),
    ("+593", "EC", "Ecuador (+593)"),
    ("+594", "GF", "French Guiana (+594)"),
    ("+595", "PY", "Paraguay (+595)"),
    ("+596", "MQ", "Martinique (+596)"),
    ("+597", "SR", "Suriname (+597)"),
    ("+598", "UY", "Uruguay (+598)"),
    ("+599", "BQ CW", "Caribbean Netherlands / Curaçao (+599)"),
    ("+670", "TL", "Timor-Leste (+670)"),
    ("+672", "NF AQ", "Australian External Territories (+672)"),
    ("+673", "BN", "Brunei (+673)"),
    ("+674", "NR", "Nauru (+674)"),
    ("+675", "PG", "Papua New Guinea (+675)"),
    ("+676", "TO", "Tonga (+676)"),
    ("+677", "SB", "Solomon Islands (+677)"),
    ("+678", "VU", "Vanuatu (+678)"),
    ("+679", "FJ", "Fiji (+679)"),
    ("+680", "PW", "Palau (+680)"),
    ("+681", "WF", "Wallis and Futuna (+681)"),
    ("+682", "CK", "Cook Islands (+682)"),
    ("+683", "NU", "Niue (+683)"),
    ("+685", "WS", "Samoa (+685)"),
    ("+686", "KI", "Kiribati (+686)"),
    ("+687", "NC", "New Caledonia (+687)"),
    ("+688", "TV", "Tuvalu (+688)"),
    ("+689", "PF", "French Polynesia (+689)"),
    ("+690", "TK", "Tokelau (+690)"),
    ("+691", "FM", "Micronesia (+691)"),
    ("+692", "MH", "Marshall Islands (+692)"),
    ("+850", "KP", "North Korea (+850)"),
    ("+852", "HK", "Hong Kong (+852)"),
    ("+853", "MO", "Macau (+853)"),
    ("+855", "KH", "Cambodia (+855)"),
    ("+856", "LA", "Laos (+856)"),
    ("+880", "BD", "Bangladesh (+880)"),
    ("+886", "TW", "Taiwan (+886)"),
    ("+960", "MV", "Maldives (+960)"),
    ("+961", "LB", "Lebanon (+961)"),
    ("+962", "JO", "Jordan (+962)"),
    ("+963", "SY", "Syria (+963)"),
    ("+964", "IQ", "Iraq (+964)"),
    ("+965", "KW", "Kuwait (+965)"),
    ("+966", "SA", "Saudi Arabia (+966)"),
    ("+967", "YE", "Yemen (+967)"),
    ("+968", "OM", "Oman (+968)"),
    ("+970", "PS", "Palestine (+970)"),
    ("+971", "AE", "United Arab Emirates (+971)"),
    ("+972", "IL", "Israel (+972)"),
    ("+973", "BH", "Bahrain (+973)"),
    ("+974", "QA", "Qatar (+974)"),
    ("+975", "BT", "Bhutan (+975)"),
    ("+976", "MN", "Mongolia (+976)"),
    ("+977", "NP", "Nepal (+977)"),
    ("+992", "TJ", "Tajikistan (+992)"),
    ("+993", "TM", "Turkmenistan (+993)"),
    ("+994", "AZ", "Azerbaijan (+994)"),
    ("+995", "GE", "Georgia (+995)"),
    ("+996", "KG", "Kyrgyzstan (+996)"),
    ("+998", "UZ", "Uzbekistan (+998)"),
];

pub fn is_valid_country_code(code: &str) -> bool {
    COUNTRY_CODES
        .iter()
        .any(|(value, _, _)| *value == code.trim())
}

/// Normalise a guest-entered number to E.164, using `default_country_code` for
/// numbers written in national form. Returns `None` when the result could not
/// possibly be a phone number, in which case the booking is rejected with a
/// validation error rather than silently losing the number.
///
/// * `+33612345678`, `+33 6 12 34 56 78` stay as they are.
/// * `0033612345678` becomes `+33612345678` (international prefix).
/// * `0612345678` becomes `+33612345678` (one trunk `0` dropped).
/// * `612345678` becomes `+33612345678` (countries with no trunk prefix).
pub fn normalize(raw: &str, default_country_code: &str) -> Option<String> {
    let cleaned: String = raw
        .trim()
        .chars()
        .filter(|c| !matches!(c, ' ' | '-' | '(' | ')' | '.' | '/' | '\u{a0}'))
        .collect();

    if cleaned.is_empty() {
        return None;
    }

    let country = default_country_code.trim();
    let normalized = if let Some(rest) = cleaned.strip_prefix("00") {
        format!("+{}", rest)
    } else if cleaned.starts_with('+') {
        cleaned
    } else {
        if !is_valid_country_code(country) {
            return None;
        }
        // A single leading zero is the national trunk prefix and is dropped;
        // countries without one (the US, say) are unaffected.
        let national = cleaned.strip_prefix('0').unwrap_or(&cleaned);
        format!("{}{}", country, national)
    };

    is_e164(&normalized).then_some(normalized)
}

/// Loose E.164 check: a leading `+` then 8 to 15 digits.
///
/// E.164 caps the total at 15 digits; the lower bound only rejects input that
/// is obviously not a number. The gateway remains the authority on validity.
pub fn is_e164(raw: &str) -> bool {
    let raw = raw.trim();
    let Some(digits) = raw.strip_prefix('+') else {
        return false;
    };
    digits.len() >= 8 && digits.len() <= 15 && digits.chars().all(|c| c.is_ascii_digit())
}

/// E.164 without the leading `+`, the wire format GatewayAPI and seven.io
/// document for recipients.
pub fn without_plus(e164: &str) -> String {
    e164.trim().trim_start_matches('+').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plausible_e164() {
        assert!(is_e164("+15551234567"));
        assert!(is_e164(" +447911123456 "));
    }

    #[test]
    fn rejects_missing_plus_or_bad_length_or_non_digits() {
        assert!(!is_e164("07911123456"));
        assert!(!is_e164("+1234"));
        assert!(!is_e164("+1555abc4567"));
        assert!(!is_e164(""));
        // 16 digits is one past the E.164 ceiling.
        assert!(!is_e164("+1234567890123456"));
    }

    #[test]
    fn normalizes_national_numbers_with_the_default_country() {
        assert_eq!(
            normalize("06 12 34 56 78", "+33").as_deref(),
            Some("+33612345678")
        );
        // No trunk prefix: nothing is dropped.
        assert_eq!(
            normalize("5551234567", "+1").as_deref(),
            Some("+15551234567")
        );
    }

    #[test]
    fn preserves_international_forms() {
        assert_eq!(
            normalize("+33 6 12 34 56 78", "+1").as_deref(),
            Some("+33612345678")
        );
        assert_eq!(
            normalize("0033612345678", "+1").as_deref(),
            Some("+33612345678")
        );
    }

    #[test]
    fn strips_punctuation_guests_actually_type() {
        assert_eq!(
            normalize("(06) 12-34.56/78", "+33").as_deref(),
            Some("+33612345678")
        );
    }

    #[test]
    fn rejects_junk() {
        assert_eq!(normalize("", "+33"), None);
        assert_eq!(normalize("not a phone", "+33"), None);
        assert_eq!(normalize("0612345678", "+999"), None);
        // Only one trunk zero is dropped, so this stays too long to be E.164.
        assert_eq!(normalize("+00000000000000000", "+33"), None);
    }

    #[test]
    fn country_code_table_is_usable() {
        assert!(is_valid_country_code("+33"));
        assert!(is_valid_country_code(" +1 "));
        assert!(!is_valid_country_code("33"));
        assert!(is_valid_country_code(DEFAULT_COUNTRY_CODE));
    }

    #[test]
    fn without_plus_matches_gateway_wire_format() {
        assert_eq!(without_plus("+33612345678"), "33612345678");
    }
}
