use fust_i18n::t;

fn main() {
    let count = 5;

    // 递归翻译：

    let msg = t!(
        "item-made",
        count,
        t!(
            "recipe-info",
            t!("iron-plate")
            // t!("item-with-quality", t!("iron-plate"), t!("legendary"))
        )
    );

    println!("{}", msg);
}
