pub trait FustServer<T> {
    fn get_template(&self, key: &'static str) -> Option<T>;
}

pub trait FustTemplate {
    fn format(&self, args: &[String]) -> String;
}

#[macro_export]
macro_rules! t {
    // 基础用法：t!("key")
    ($key:expr $(,)?) => {
        $crate::translate($key, &[])
    };

    // 进阶用法：t!("key", arg1, arg2, ...)
    // 这里的 arg 会被递归 format!，所以嵌套 t!() 完美运行
    ($key:expr, $($arg:expr),+ $(,)?) => {
        $crate::translate($key, &[
            $( $arg.to_string() ),+
        ])
    };
}

pub fn translate(key: &'static str, args: &[String]) -> String {
    let template = get_template(key); // 从你的本地化配置中获取

    // 高效渲染：避免多次 replace 产生中间 String
    let mut result = String::with_capacity(template.len() + args.len() * 10);
    let mut last_end = 0;

    // 简单的正则或字符串扫描逻辑，寻找 __1__, __2__
    // 这里用伪代码示意核心逻辑
    for (i, arg) in args.iter().enumerate() {
        let placeholder = format!("__{}__", i + 1);
        // 实际开发建议用字符串索引扫描，这里为了演示逻辑：
        if let Some(pos) = template.find(&placeholder) {
            result.push_str(&template[last_end..pos]);
            result.push_str(arg);
            last_end = pos + placeholder.len();
        }
    }
    result.push_str(&template[last_end..]);
    result
}

fn get_template(key: &'static str) -> &'static str {
    match key {
        "item-made" => "成功制作了 __1__ 个 __2__。",
        "iron-plate" => "铁板",
        "recipe-info" => "[物品: __1__]",
        "quality" => "品质: __1__",
        "normal" => "普通",
        "legendary" => "传奇",
        "item-with-quality" => "[物品: __1__, 品质: __2__]",
        _ => key,
    }
}
