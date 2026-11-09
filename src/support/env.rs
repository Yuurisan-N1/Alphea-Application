use anyhow::{anyhow, Context, Result};

#[derive(Clone)]
pub struct Account {
    pub email: String,
}

pub fn load(path: &str) -> Result<Vec<Account>> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("the account file {} could not be read", path))?;
    let mut out = Vec::new();
    for line in raw.lines() {
        let text = line.trim();
        if text.is_empty() {
            continue;
        }
        if !text.ends_with("@gmail.com") {
            return Err(anyhow!(
                "the account file holds a line that is not a gmail address"
            ));
        }
        out.push(Account {
            email: text.to_string(),
        });
    }
    out.sort_by(|a, b| a.email.cmp(&b.email));
    out.dedup_by(|a, b| a.email == b.email);
    Ok(out)
}
