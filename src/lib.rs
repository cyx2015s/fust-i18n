static LOCALE: std::sync::LazyLock<std::sync::Arc<std::sync::RwLock<String>>> =
    std::sync::LazyLock::new(|| std::sync::Arc::new(std::sync::RwLock::new(String::from("zh-CN"))));

pub fn set_locale(locale: &str) {
    let mut loc = LOCALE.write().unwrap();
    *loc = locale.to_string();
}

pub fn get_locale() -> String {
    let loc = LOCALE.read().unwrap();
    loc.clone()
}

pub type I18nDict = std::collections::HashMap<String, String, ahash::RandomState>;

pub type I18nDicts = std::collections::HashMap<String, I18nDict, ahash::RandomState>;

static I18N_DICTS: std::sync::LazyLock<std::sync::Arc<std::sync::RwLock<I18nDicts>>> =
    std::sync::LazyLock::new(|| std::sync::Arc::new(std::sync::RwLock::new(I18nDicts::default())));

pub fn parse_ini<R: std::io::Read>(mut reader: R) -> Result<I18nDict, ini::Error> {
    let file = ini::Ini::read_from(&mut reader).unwrap();
    let mut dict = I18nDict::default();
    for (sec, prop) in file.iter() {
        if let Some(sec) = sec {
            for (k, v) in prop.iter() {
                dict.insert(sec.to_string() + "." + k, v.to_string());
            }
        } else {
            for (k, v) in prop.iter() {
                dict.insert(k.to_string(), v.to_string());
            }
        }
    }
    Ok(dict)
}

pub fn reset_i18n_dicts() {
    let mut dicts = I18N_DICTS.write().unwrap();
    *dicts = I18nDicts::default();
}

pub fn update_i18n_dicts(locale: &str, dict: I18nDict) {
    let mut dicts = I18N_DICTS.write().unwrap();
    if let Some(old_dict) = dicts.get_mut(locale) {
        old_dict.extend(dict);
    } else {
        dicts.insert(locale.to_string(), dict);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalisedString {
    Literal(String),
    Function(Vec<LocalisedString>),
}

impl From<&str> for LocalisedString {
    fn from(value: &str) -> Self {
        LocalisedString::Literal(value.to_string())
    }
}

impl From<Vec<&str>> for LocalisedString {
    fn from(value: Vec<&str>) -> Self {
        assert!(!value.is_empty(), "Function cannot be empty");
        LocalisedString::Function(
            value
                .into_iter()
                .map(|x| LocalisedString::Literal(x.to_string()))
                .collect(),
        )
    }
}

impl From<String> for LocalisedString {
    fn from(value: String) -> Self {
        LocalisedString::Literal(value)
    }
}

impl From<Vec<String>> for LocalisedString {
    fn from(value: Vec<String>) -> Self {
        assert!(!value.is_empty(), "Function cannot be empty");
        LocalisedString::Function(value.into_iter().map(LocalisedString::Literal).collect())
    }
}

impl From<Vec<LocalisedString>> for LocalisedString {
    fn from(value: Vec<LocalisedString>) -> Self {
        assert!(!value.is_empty(), "Function cannot be empty");
        LocalisedString::Function(value)
    }
}

#[macro_export]
macro_rules! t {
    ($key:expr $(,)?) => {
        LocalisedString::from(vec![$key])
    };
    ($key:expr, $($arg:expr),* $(,)?) => {
        LocalisedString::from(vec![
            LocalisedString::from($key),
            $(
                LocalisedString::from($arg),
            )*
        ])
    }
}

static PARAM_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"__([1-9]\d*)__").unwrap());

// 实现 Display，打印出来就是翻译好的结果
impl std::fmt::Display for LocalisedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_unknown_key(f: &mut std::fmt::Formatter<'_>, key: &str) -> std::fmt::Result {
            f.write_str("Unknown key: ").unwrap();
            f.write_str(key)
        }
        match self {
            LocalisedString::Literal(s) => f.write_str(s).unwrap(),
            LocalisedString::Function(vec) => {
                let dicts = I18N_DICTS.read().unwrap();
                if let Some(dict) = dicts.get(&get_locale()) {
                    if let Some(value) = dict.get(match &vec[0] {
                        LocalisedString::Literal(s) => s,
                        _ => return write_unknown_key(f, "Invalid key format"),
                    }) {
                        // 将__1__、__2__等占位符替换为参数
                        let mut offset = 0;
                        for cap in PARAM_REGEX.captures_iter(value) {
                            let whole_match = cap.get(0).unwrap();
                            f.write_str(&value[offset..whole_match.start()])?;
                            offset = whole_match.end();
                            let index = cap[1].parse::<usize>().unwrap();
                            if index < vec.len() {
                                f.write_str(&vec[index].to_string())?;
                            } else {
                                f.write_str(whole_match.as_str())?;
                            }
                        }
                        f.write_str(&value[offset..])?;
                    } else {
                        write_unknown_key(
                            f,
                            match &vec[0] {
                                LocalisedString::Literal(s) => s,
                                _ => return write_unknown_key(f, "Invalid key format"),
                            },
                        )
                        .unwrap();
                    }
                } else {
                    write_unknown_key(
                        f,
                        match &vec[0] {
                            LocalisedString::Literal(s) => s,
                            _ => return write_unknown_key(f, "Invalid key format"),
                        },
                    )
                    .unwrap();
                }
            }
        }
        Ok(())
    }
}

#[test]
fn test_macro() {
    let s1 = t!("hello");
    let s2 = t!("section.key", "param1", "param2");
    let s3 = t!("nested", t!("inner.key", "inner.param"), "outer.param");
    assert_eq!(
        s1,
        LocalisedString::Function(vec![LocalisedString::Literal("hello".to_string())])
    );
    assert_eq!(
        s2,
        LocalisedString::Function(vec![
            LocalisedString::Literal("section.key".to_string()),
            LocalisedString::Literal("param1".to_string()),
            LocalisedString::Literal("param2".to_string()),
        ])
    );
    assert_eq!(
        s3,
        LocalisedString::Function(vec![
            LocalisedString::Literal("nested".to_string()),
            LocalisedString::Function(vec![
                LocalisedString::Literal("inner.key".to_string()),
                LocalisedString::Literal("inner.param".to_string()),
            ]),
            LocalisedString::Literal("outer.param".to_string()),
        ])
    );
}

#[test]
fn test_translate() {
    update_i18n_dicts(
        "zh-CN",
        parse_ini(std::io::Cursor::new(include_str!("../assets/base.cfg"))).unwrap(),
    );
    update_i18n_dicts(
        "zh-CN",
        parse_ini(std::io::Cursor::new(include_str!("../assets/core.cfg"))).unwrap(),
    );
    update_i18n_dicts(
        "zh-CN",
        I18nDict::from_iter(
            [(
                "malformed-key".to_string(),
                "__0___1__ __2__ _____3_____ __4_".to_string(),
            )]
            .into_iter(),
        ),
    );
    set_locale("zh-CN");
    let s = t!(
        "changed-filter",
        "ferris",
        "1",
        t!("item-name.iron-plate"),
        "2",
        t!("item-name.copper-plate")
    );
    eprintln!("{:#?}", &s);
    eprintln!("{}", &s);
    eprintln!("{}", t!("malformed-key", "a", "b"));
    assert_eq!(
        s.to_string(),
        "ferris 将 1 份的 铁板 请求改为 2 份的 铜板".to_string()
    );
}
