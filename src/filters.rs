pub fn color(val: &f64) -> ::askama::Result<String> {
    let color = if *val < 32.0 {
        "white"
    } else if *val < 68.0 {
        "cyan"
    } else if *val < 79.0 {
        "#99ff00"
    } else if *val < 90.0 {
        "orange"
    } else {
        "red"
    };

    Ok(color.to_string())
}
