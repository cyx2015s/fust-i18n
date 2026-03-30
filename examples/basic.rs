use fust_i18n::{set_locale, update_i18n_ini};

fn main() {
    set_locale("zh-CN");
    update_i18n_ini(
        "zh-CN",
        std::fs::OpenOptions::new()
            .read(true)
            .open("./assets/base.cfg")
            .unwrap(),
    )
    .unwrap();
    println!("{}", fust_i18n::t!("item-name.iron-plate"));
}
