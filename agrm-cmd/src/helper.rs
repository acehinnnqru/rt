const REPONAME_REGEX: &str = r"^[a-zA-Z0-9\-_\.]+$";
const NS_AND_NAME_REGEX: &str = r"^[a-zA-Z0-9\-_\.]+/[a-zA-Z0-9\-_\.]+$";

// return Vector of (namespace, name)
pub fn parse_repositories(namespace: Option<String>, repos: Vec<String>) -> Vec<(String, String)> {
    let mut v = Vec::<(String, String)>::new();
    let regex_str = match namespace {
        Some(_) => REPONAME_REGEX.to_string(),
        None => NS_AND_NAME_REGEX.to_string(),
    };

    let re = regex::Regex::new(&regex_str).unwrap();

    repos
        .iter()
        .filter_map(|r| {
            if !re.is_match(r) {
                eprintln!("Invalid repository name: {}", r);
                return None;
            }
            Some(match &namespace {
                Some(ns) => (ns.to_string(), r.to_string()),
                None => {
                    let v: Vec<&str> = r.split('/').collect();
                    (v[0].to_string(), v[1].to_string())
                }
            })
        })
        .for_each(|r| v.push(r));
    v
}
