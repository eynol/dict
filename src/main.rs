
use console::{style, Attribute};
use std::env;
mod yodaodict;

struct CliOptions {
    debug: bool,
}

static mut DEBUG_MODE: bool = false;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let tt = vec![1, 2, 3, 4, 5];
    // test(tt);

    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let debug_index = args.binary_search(&String::from("--debug"));

    let mut options = CliOptions { debug: false };
    if let Ok(idx) = debug_index {
        args.remove(idx);
        options.debug = true;
    }

    if args.len() < 1 {
        println!("Usage: {} <单词>", args[0]);
        return Ok(());
    }

    let word = args.get(0).unwrap();
    if word == "-v" || word == "--version" {
        //  只查看version
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        println!("v{}", VERSION);
        return Ok(());
    }

    let url = yodaodict::build_query_url(word);
    unsafe {
        if options.debug {
            DEBUG_MODE = options.debug;
            let url = style(&url).attr(Attribute::Underlined);
            println!("url is {url}",);
        }
    }

    let resp = reqwest::blocking::get(url)?.text()?;

    if options.debug {
        println!(" xml data:{} \n", &resp)
    }
    let xml_node_list = yodaodict::get_xml_node_list(&resp);
    if options.debug {
        for (paths, value) in xml_node_list.iter() {
            println!("{}: {}", paths.join("/").as_str(), style(value).green());
        }
    }

    print_basic_info(&xml_node_list);

    print_tanslation_content(&xml_node_list);
    print_web_tanslation_content(&xml_node_list);
    print_wiki(&xml_node_list);
    // println!("{:#?}", resp);
    Ok(())
}

fn get_key_value<'a>(
    xml_node_list: &'a Vec<(Vec<String>, String)>,
    key: &str,
) -> Option<&'a (Vec<String>, String)> {
    xml_node_list.iter().find(|x| {
        if let Some(k) = x.0.get(0) {
            return k.eq(key);
        }
        false
    })
}

fn filter_prefix<'a, 'b>(
    xml_node_list: &'a Vec<(Vec<String>, String)>,
    keys: &'b Vec<String>,
) -> Option<Vec<&'a (Vec<String>, String)>> {
    let list: Vec<&'a (Vec<String>, String)> = xml_node_list
        .iter()
        .filter(|x| x.0.starts_with(keys))
        .collect();
    if list.len() > 0 {
        return Some(list);
    }
    None
}

// 打印基础信息
fn print_basic_info(xml_node_list: &Vec<(Vec<String>, String)>) {
    if let Some((_, value)) = get_key_value(xml_node_list, "return-phrase") {
        print!("  {}", style(value).bold().green());
    }

    if let Some((_, value)) = get_key_value(xml_node_list, "phonetic-symbol") {
        print!("{}", style(format!("  [{}]", value)).bold().green().dim());
    }
    print!("\n")
}

// 打印翻译结果
fn print_tanslation_content(xml_node_list: &Vec<(Vec<String>, String)>) {
    let translation_keys = ["custom-translation", "translation", "content"]
        .map(String::from)
        .to_vec();

    if let Some(list) = filter_prefix(xml_node_list, &translation_keys) {
        print_section_title("翻译");
        for (_, value) in list.iter() {
            println!("  {}", value);
        }
        print!("\n")
    }
}

// 打印网络翻译结果
fn print_web_tanslation_content(xml_node_list: &Vec<(Vec<String>, String)>) {
    let translate_keys = ["yodao-web-dict", "web-translation"]
        .map(String::from)
        .to_vec();

    if let Some(list) = filter_prefix(xml_node_list, &translate_keys) {
        print_section_title("网络翻译");
        let mut print_next_line = false;
        for (paths, value) in list.iter() {
            match paths.last() {
                Some(last_word) if last_word.eq("key") => {
                    if !print_next_line {
                        print_next_line = true;
                    } else {
                        print!("\n");
                    }
                    print!("{} \n  ", style(value).magenta());
                }
                Some(last_word) if last_word.eq("cl") => (),
                _ => {
                    print!("{}; ", value);
                }
            }
        }
        print!("\n")
    }
}

fn print_section_title(title: &str) {
    println!(
        "========{txt:^width$}========",
        txt = style(title).magenta().dim(),
        width = 8
    );
}
// 打印 Wiki 结果
fn print_wiki(xml_node_list: &Vec<(Vec<String>, String)>) {
    let wiki_keys = [
        "wand-result",
        "wand",
        "entity",
        "wiki-entry",
        "info",
        "data",
    ]
    .map(String::from)
    .to_vec();

    if let Some(list) = filter_prefix(xml_node_list, &wiki_keys) {
        print_section_title("Wiki");
        for (paths, value) in list.iter() {
            match paths.last() {
                Some(word) if word.eq("entry") => {
                    println!("{}", style(format!("[ {} ]", value)).green())
                }
                Some(word) if word.eq("wikiType") => (),
                Some(word) if word.eq("summary") => println!("  {}", value),
                _ => (),
            }
        }
        print!("\n")
    }
}
