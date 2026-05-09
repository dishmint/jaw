pub struct LangSpec {
    pub line_prefix: &'static str,
    pub block: Option<BlockComment>,
}

pub struct BlockComment {
    pub open: &'static str,
    pub close: &'static str,
}

pub fn lookup(name: &str) -> Option<&'static LangSpec> {
    LANGS
        .iter()
        .find(|(aliases, _)| aliases.iter().any(|a| a.eq_ignore_ascii_case(name)))
        .map(|(_, spec)| spec)
}

pub fn known_languages() -> Vec<&'static str> {
    LANGS.iter().map(|(aliases, _)| aliases[0]).collect()
}

const C_STYLE: LangSpec = LangSpec {
    line_prefix: "//",
    block: Some(BlockComment {
        open: "/*",
        close: "*/",
    }),
};

const HASH_STYLE: LangSpec = LangSpec {
    line_prefix: "#",
    block: None,
};

const LANGS: &[(&[&str], LangSpec)] = &[
    (&["python", "py"], HASH_STYLE),
    (&["ruby", "rb"], HASH_STYLE),
    (&["bash", "sh", "shell"], HASH_STYLE),
    (
        &["rust", "rs"],
        LangSpec {
            line_prefix: "//",
            block: Some(BlockComment {
                open: "/*",
                close: "*/",
            }),
        },
    ),
    (&["javascript", "js"], C_STYLE),
    (&["typescript", "ts"], C_STYLE),
    (&["go"], C_STYLE),
    (&["c"], C_STYLE),
    (&["cpp", "c++", "cxx"], C_STYLE),
    (
        &["lua"],
        LangSpec {
            line_prefix: "--",
            block: Some(BlockComment {
                open: "--[[",
                close: "]]",
            }),
        },
    ),
    (
        &["sql"],
        LangSpec {
            line_prefix: "--",
            block: None,
        },
    ),
];
