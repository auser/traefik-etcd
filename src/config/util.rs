use super::{RuleConfig, SelectionConfig};

pub fn add_selection_rules<T>(with_selection: &T, rules: &mut RuleConfig)
where
    T: Into<Option<SelectionConfig>> + Clone,
{
    let selection_rules: Option<SelectionConfig> = (*with_selection).clone().into();
    if let Some(selection) = &selection_rules {
        if let Some(with_cookie) = &selection.with_cookie {
            rules.add_header_rule(
                "Cookie",
                &format!(
                    "{}={}",
                    with_cookie.name,
                    with_cookie.value.as_deref().unwrap_or("true")
                ),
            );
        }
        if let Some(from_client_ip) = &selection.from_client_ip {
            rules.add_client_ip_rule(
                from_client_ip.ip.as_deref(),
                from_client_ip.range.as_deref(),
            );
        }
    }
}
