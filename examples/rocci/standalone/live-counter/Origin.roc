Origin := [].{
    Place : {
        country : Str,
        city : Str,
        region : Str,
        flag : Str,
    }

    lookup : Str -> Place
    lookup = |tz|
        match tz {
            "America/New_York" => { country: "US", city: "New York", region: "United States", flag: "US" }
            "America/Chicago" => { country: "US", city: "Chicago", region: "United States", flag: "US" }
            "America/Denver" => { country: "US", city: "Denver", region: "United States", flag: "US" }
            "America/Los_Angeles" => { country: "US", city: "Los Angeles", region: "United States", flag: "US" }
            "America/Toronto" => { country: "CA", city: "Toronto", region: "Canada", flag: "CA" }
            "America/Vancouver" => { country: "CA", city: "Vancouver", region: "Canada", flag: "CA" }
            "America/Sao_Paulo" => { country: "BR", city: "Sao Paulo", region: "Brazil", flag: "BR" }
            "America/Mexico_City" => { country: "MX", city: "Mexico City", region: "Mexico", flag: "MX" }
            "Europe/London" => { country: "GB", city: "London", region: "United Kingdom", flag: "GB" }
            "Europe/Dublin" => { country: "IE", city: "Dublin", region: "Ireland", flag: "IE" }
            "Europe/Paris" => { country: "FR", city: "Paris", region: "France", flag: "FR" }
            "Europe/Berlin" => { country: "DE", city: "Berlin", region: "Germany", flag: "DE" }
            "Europe/Amsterdam" => { country: "NL", city: "Amsterdam", region: "Netherlands", flag: "NL" }
            "Europe/Stockholm" => { country: "SE", city: "Stockholm", region: "Sweden", flag: "SE" }
            "Europe/Oslo" => { country: "NO", city: "Oslo", region: "Norway", flag: "NO" }
            "Europe/Copenhagen" => { country: "DK", city: "Copenhagen", region: "Denmark", flag: "DK" }
            "Europe/Helsinki" => { country: "FI", city: "Helsinki", region: "Finland", flag: "FI" }
            "Europe/Madrid" => { country: "ES", city: "Madrid", region: "Spain", flag: "ES" }
            "Europe/Rome" => { country: "IT", city: "Rome", region: "Italy", flag: "IT" }
            "Europe/Warsaw" => { country: "PL", city: "Warsaw", region: "Poland", flag: "PL" }
            "Europe/Zurich" => { country: "CH", city: "Zurich", region: "Switzerland", flag: "CH" }
            "Europe/Vienna" => { country: "AT", city: "Vienna", region: "Austria", flag: "AT" }
            "Europe/Prague" => { country: "CZ", city: "Prague", region: "Czechia", flag: "CZ" }
            "Europe/Lisbon" => { country: "PT", city: "Lisbon", region: "Portugal", flag: "PT" }
            "Europe/Athens" => { country: "GR", city: "Athens", region: "Greece", flag: "GR" }
            "Europe/Moscow" => { country: "RU", city: "Moscow", region: "Russia", flag: "RU" }
            "Africa/Cairo" => { country: "EG", city: "Cairo", region: "Egypt", flag: "EG" }
            "Africa/Johannesburg" => { country: "ZA", city: "Johannesburg", region: "South Africa", flag: "ZA" }
            "Asia/Tokyo" => { country: "JP", city: "Tokyo", region: "Japan", flag: "JP" }
            "Asia/Shanghai" => { country: "CN", city: "Shanghai", region: "China", flag: "CN" }
            "Asia/Hong_Kong" => { country: "HK", city: "Hong Kong", region: "Hong Kong", flag: "HK" }
            "Asia/Singapore" => { country: "SG", city: "Singapore", region: "Singapore", flag: "SG" }
            "Asia/Seoul" => { country: "KR", city: "Seoul", region: "South Korea", flag: "KR" }
            "Asia/Kolkata" => { country: "IN", city: "Kolkata", region: "India", flag: "IN" }
            "Asia/Dubai" => { country: "AE", city: "Dubai", region: "United Arab Emirates", flag: "AE" }
            "Asia/Bangkok" => { country: "TH", city: "Bangkok", region: "Thailand", flag: "TH" }
            "Asia/Jakarta" => { country: "ID", city: "Jakarta", region: "Indonesia", flag: "ID" }
            "Australia/Sydney" => { country: "AU", city: "Sydney", region: "Australia", flag: "AU" }
            "Australia/Melbourne" => { country: "AU", city: "Melbourne", region: "Australia", flag: "AU" }
            "Pacific/Auckland" => { country: "NZ", city: "Auckland", region: "New Zealand", flag: "NZ" }
            "UTC" => { country: "", city: "UTC", region: "Coordinated Universal Time", flag: "UTC" }
            _ => fallback(tz)
        }

    relative_ago : I64, I64 -> Str
    relative_ago = |now_secs, then_secs| {
        delta = if now_secs > then_secs { now_secs - then_secs } else { 0.I64 }
        if delta < 5.I64 {
            "just now"
        } else if delta < 60.I64 {
            "${delta.to_str()} seconds ago"
        } else if delta < 3600.I64 {
            counted_ago(delta.div_trunc_by(60.I64), "minute")
        } else if delta < 86400.I64 {
            counted_ago(delta.div_trunc_by(3600.I64), "hour")
        } else {
            counted_ago(delta.div_trunc_by(86400.I64), "day")
        }
    }

    place_label : Place -> Str
    place_label = |place|
        if place.city != "" and place.region != "" {
            "${place.city}, ${place.region}"
        } else if place.city != "" {
            place.city
        } else if place.region != "" {
            place.region
        } else {
            "Somewhere"
        }

    signal_str : Str, Str -> Str
    signal_str = |json, key| {
        needle = "\"${key}\":\""
        parts = Str.split_on(json, needle)
        match List.get(parts, 1) {
            Ok(after) =>
                match List.get(Str.split_on(after, "\""), 0) {
                    Ok(value) => value
                    Err(_) => ""
                }
            Err(_) => ""
        }
    }

    Choice : { tz : Str, label : Str }

    ## Curated places for the optional origin picker (values must match `lookup`).
    choices : List(Choice)
    choices = [
        { tz: "America/New_York", label: "New York" },
        { tz: "America/Los_Angeles", label: "Los Angeles" },
        { tz: "America/Sao_Paulo", label: "Sao Paulo" },
        { tz: "Europe/London", label: "London" },
        { tz: "Europe/Berlin", label: "Berlin" },
        { tz: "Europe/Stockholm", label: "Stockholm" },
        { tz: "Africa/Johannesburg", label: "Johannesburg" },
        { tz: "Asia/Tokyo", label: "Tokyo" },
        { tz: "Asia/Singapore", label: "Singapore" },
        { tz: "Australia/Sydney", label: "Sydney" },
        { tz: "Pacific/Auckland", label: "Auckland" },
    ]
}

counted_ago = |n, unit|
    if n == 1.I64 {
        "1 ${unit} ago"
    } else {
        "${n.to_str()} ${unit}s ago"
    }

fallback = |tz| {
    parts = Str.split_on(tz, "/")
    city =
        match List.last(parts) {
            Ok(part) if part != "" => part
            _ => "Somewhere"
        }
    { country: "", city, region: "", flag: "--" }
}
