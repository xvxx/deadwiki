/// Single Wiki Page

pub struct Page<'s> {
    pub name: &'s str,
    pub path: &'s str,
}

impl<'s> Page<'s> {
    pub fn new(name: &'s str, path: &'s str) -> Page<'s> {
        Page { name, path }
    }

    pub fn title(&self) -> String {
        self.name
            .split('_')
            .map(|part| {
                if part.contains('/') {
                    let mut parts = part.split('/').rev();
                    let last = parts.next().unwrap_or("?");
                    format!(
                        "{}/{}",
                        parts.rev().collect::<Vec<_>>().join("/"),
                        capitalize(last)
                    )
                } else {
                    capitalize(&part)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    format!(
        "{}{}",
        s.chars().next().unwrap_or('?').to_uppercase(),
        &s.chars().skip(1).collect::<String>()
    )
}
