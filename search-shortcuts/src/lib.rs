pub mod errors;
use errors::Result;

use url::Url;

fn handle_static_redirects(query: &str) -> Result<Option<Url>> {
    Ok(match query {
        "twir" => Url::parse("https://this-week-in-rust.org")?.into(),
        "abc" => Url::parse("https://www.abc.net.au/news")?.into(),
        "had" => Url::parse("https://hackaday.com/blog/")?.into(),
        "sd" => Url::parse("https://slashdot.org")?.into(),
        "/." => Url::parse("https://slashdot.org")?.into(),
        "sth" => Url::parse("https://www.servethehome.com")?.into(),
        "x" => Url::parse("https://xkcd.com")?.into(),
        "weather" => Url::parse("https://weather.bom.gov.au/location/r3dp390-canberra")?.into(),
        "gh" => Url::parse("https://github.com")?.into(),
        "bfio" => Url::parse("https://bushfire.io")?.into(),
        _ => None,
    })
}

fn handle_docs(query: &str) -> Result<Url> {
    Ok(if query == "std" {
        Url::parse("https://doc.rust-lang.org/stable/std/")?
    } else {
        let url = Url::parse("https://docs.rs/")?;
        if let Some((crate_, version)) = {
            let sep_idx = if let Some(sep_idx) = query.find('/') {
                Some(sep_idx)
            } else if let Some(sep_idx) = query.find('@') {
                Some(sep_idx)
            } else {
                None
            };
            if let Some(sep_idx) = sep_idx {
                let (crate_, version) = query.split_at(sep_idx);
                if version.len() > 1 {
                    Some((crate_, &version[1..]))
                } else {
                    None
                }
            } else {
                None
            }
        } {
            // crate + version
            url.join(&format!("{}/{}", crate_, version))?
        } else {
            // crate
            if query.ends_with('/') {
                url.join(query)?
            } else {
                url.join(&format!("{}/", query))?
            }
        }
    })
}

fn handle_github(query: &str) -> Result<Url> {
    let url = Url::parse("https://github.com/")?;
    Ok(
        if let Some(user) = {
            if let Some(query) = query.strip_prefix('@') {
                Some(query)
            } else if let Some(query) = query.strip_prefix("u/") {
                Some(query)
            } else {
                None
            }
        } {
            url.join(user)?
        } else if query.contains('/') {
            url.join(query)?
        } else {
            let mut url = url.join("search")?;
            url.query_pairs_mut().append_pair("q", query);
            url
        },
    )
}

fn handle_wikipedia(query: &str) -> Result<Url> {
    Ok(Url::parse_with_params(
        "https://en.wikipedia.org/wiki/Special:Search",
        &[("search", query)],
    )?)
}

fn handle_stackoverflow(query: &str) -> Result<Url> {
    Ok(Url::parse_with_params(
        "https://stackoverflow.com/search",
        &[("q", query)],
    )?)
}

fn handle_docker_hub(query: &str) -> Result<Url> {
    let url = Url::parse("https://hub.docker.com/")?;
    Ok(if let Some(query) = query.strip_prefix("r/") {
        if !query.contains('/') {
            url.join("_/")?.join(query)?
        } else {
            url.join("r/")?.join(query)?
        }
    } else if let Some(query) = query.strip_prefix('/') {
        url.join("_/")?.join(query)?
    } else if let Some(query) = query.strip_prefix("_/") {
        url.join("_/")?.join(query)?
    } else if query.contains('/') {
        url.join("r/")?.join(query)?
    } else {
        let mut url = url.join("search")?;
        url.query_pairs_mut().append_pair("q", query);
        url
    })
}

fn handle_crates(query: &str) -> Result<Url> {
    Ok(Url::parse_with_params(
        "https://crates.io/search",
        &[("q", query)],
    )?)
}

fn handle_auspost(query: &str) -> Result<Url> {
    Ok(Url::parse(&format!(
        "https://auspost.com.au/mypost/track/#/details/{}",
        query
    ))?)
}

pub fn query_to_url(query: &str) -> Result<Url> {
    if let Some(url) = handle_static_redirects(query)? {
        return Ok(url);
    }
    if let Some(query) = query.strip_prefix("docs ") {
        return Ok(handle_docs(query)?);
    }
    if let Some(query) = query.strip_prefix("docs/") {
        return Ok(handle_docs(query)?);
    }
    if let Some(query) = query.strip_prefix("gh ") {
        return Ok(handle_github(query)?);
    }
    if let Some(query) = query.strip_prefix("w ") {
        return Ok(handle_wikipedia(query)?);
    }
    if let Some(query) = query.strip_prefix("so ") {
        return Ok(handle_stackoverflow(query)?);
    }
    if let Some(query) = query.strip_prefix("dh ") {
        return Ok(handle_docker_hub(query)?);
    }
    if let Some(query) = query.strip_prefix("crates ") {
        return Ok(handle_crates(query)?);
    }
    if let Some(query) = query.strip_prefix("ap ") {
        return Ok(handle_auspost(query)?);
    }
    Ok(Url::parse_with_params(
        "https://duckduckgo.com/?k1=-1",
        &[("q", query)],
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_tests(tests: &[(&str, &str)]) -> Result<()> {
        for (expected, query) in tests.iter() {
            let actual = query_to_url(query)?;
            assert_eq!(*expected, actual.as_str(), "query: {:?}", query);
        }
        Ok(())
    }

    #[test]
    fn test_github_user() -> Result<()> {
        let tests = [
            ("https://github.com/fullylegit", "gh @fullylegit"),
            ("https://github.com/fullylegit", "gh u/fullylegit"),
        ];
        run_tests(&tests)
    }

    #[test]
    fn test_github_repo() -> Result<()> {
        let tests = [("https://github.com/fullylegit/ja3", "gh fullylegit/ja3")];
        run_tests(&tests)
    }

    #[test]
    fn test_github_search() -> Result<()> {
        let tests = [("https://github.com/search?q=test", "gh test")];
        run_tests(&tests)
    }

    #[test]
    fn test_docker_hub_repo() -> Result<()> {
        let tests = [
            ("https://hub.docker.com/_/nginx", "dh /nginx"),
            ("https://hub.docker.com/_/nginx", "dh _/nginx"),
            ("https://hub.docker.com/_/nginx", "dh r/nginx"),
            (
                "https://hub.docker.com/r/nvidia/k8s-device-plugin",
                "dh nvidia/k8s-device-plugin",
            ),
            (
                "https://hub.docker.com/r/nvidia/k8s-device-plugin",
                "dh r/nvidia/k8s-device-plugin",
            ),
        ];
        run_tests(&tests)
    }

    #[test]
    fn test_docker_hub_search() -> Result<()> {
        let tests = [("https://hub.docker.com/search?q=test", "dh test")];
        run_tests(&tests)
    }

    #[test]
    fn test_docs_crate() -> Result<()> {
        let tests = [
            ("https://docs.rs/actix-web/", "docs actix-web"),
            ("https://docs.rs/actix-web/", "docs/actix-web"),
        ];
        run_tests(&tests)
    }

    #[test]
    fn test_docs_crate_version() -> Result<()> {
        let tests = [
            ("https://docs.rs/actix-web/3.3.0", "docs actix-web/3.3.0"),
            ("https://docs.rs/actix-web/3.3.0", "docs/actix-web/3.3.0"),
            ("https://docs.rs/actix-web/3.3.0", "docs actix-web@3.3.0"),
            ("https://docs.rs/actix-web/3.3.0", "docs/actix-web@3.3.0"),
        ];
        run_tests(&tests)
    }

    #[test]
    fn test_docs_std() -> Result<()> {
        // this is already handled by docs.rs but doing it here removes
        // an additional redirect
        let tests = [
            ("https://doc.rust-lang.org/stable/std/", "docs std"),
            ("https://doc.rust-lang.org/stable/std/", "docs/std"),
        ];
        run_tests(&tests)
    }

    #[test]
    fn test_search() -> Result<()> {
        let tests = [
            ("https://duckduckgo.com/?k1=-1&q=search", "search"),
            ("https://duckduckgo.com/?k1=-1&q=lol+donkey", "lol donkey"),
            ("https://duckduckgo.com/?k1=-1&q=lol%2Fdonkey", "lol/donkey"),
        ];
        run_tests(&tests)
    }

    #[test]
    fn test_static_shortcuts() -> Result<()> {
        let tests = [
            ("https://this-week-in-rust.org/", "twir"),
            ("https://www.abc.net.au/news", "abc"),
            ("https://hackaday.com/blog/", "had"),
            ("https://slashdot.org/", "sd"),
            ("https://slashdot.org/", "/."),
            ("https://www.servethehome.com/", "sth"),
            ("https://xkcd.com/", "x"),
            (
                "https://weather.bom.gov.au/location/r3dp390-canberra",
                "weather",
            ),
            ("https://github.com/", "gh"),
            ("https://bushfire.io/", "bfio"),
        ];
        run_tests(&tests)
    }

    #[test]
    fn test_wikipedia_search() -> Result<()> {
        let tests = [
            (
                "https://en.wikipedia.org/wiki/Special:Search?search=test",
                "w test",
            ),
            (
                "https://en.wikipedia.org/wiki/Special:Search?search=test%2Flol",
                "w test/lol",
            ),
        ];
        run_tests(&tests)
    }

    #[test]
    fn test_stackoverflow_search() -> Result<()> {
        let tests = [
            ("https://stackoverflow.com/search?q=search", "so search"),
            (
                "https://stackoverflow.com/search?q=lol+donkey",
                "so lol donkey",
            ),
            (
                "https://stackoverflow.com/search?q=lol%2Fdonkey",
                "so lol/donkey",
            ),
        ];
        run_tests(&tests)
    }

    #[test]
    fn test_crates_search() -> Result<()> {
        let tests = [
            ("https://crates.io/search?q=search", "crates search"),
            ("https://crates.io/search?q=lol+donkey", "crates lol donkey"),
            (
                "https://crates.io/search?q=lol%2Fdonkey",
                "crates lol/donkey",
            ),
        ];
        run_tests(&tests)
    }

    #[test]
    fn test_auspost_tracking() -> Result<()> {
        let tests = [(
            "https://auspost.com.au/mypost/track/#/details/ABC123",
            "ap ABC123",
        )];
        run_tests(&tests)
    }
}
