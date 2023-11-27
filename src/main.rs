use console::{style, Attribute};
use yodaodict::DictResult;
mod yodaodict;
use std::env;

struct CliOptions {
    debug: bool,
}

static mut DEBUG_MODE: bool = false;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let tt = vec![1, 2, 3, 4, 5];
    // test(tt);

    let mut args: Vec<String> = env::args().skip(1).collect();
    // args.remove(0);

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

    // let word: &String = &args[1];
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
    let dict_rs = yodaodict::get_xml_node_list(&word, &resp);
    if options.debug {
        println!("xml is {} \n", resp);
        println!("dict rs is {:#?}", dict_rs);
    }

    print_basic_info(&dict_rs);

    print_tanslation_content(&dict_rs);
    print_web_tanslation_content(&dict_rs);
    print_wiki(&dict_rs);
    // println!("{:#?}", resp);
    Ok(())
}

// 打印基础信息
fn print_basic_info(xml_node_list: &DictResult) {
    if let Some(value) = &xml_node_list.return_phrase {
        print!("  {}", style(value).bold().green());
    }

    if let Some(value) = &xml_node_list.phonetic_symbol {
        print!("{}", style(format!("  [{}]", value)).bold().green().dim());
    }
    print!("\n")
}

// 打印翻译结果
fn print_tanslation_content(xml_node_list: &DictResult) {
    if let Some(trans) = &xml_node_list.translation {
        print_section_title("翻译");
        for value in trans.content.iter() {
            println!("  {}", value);
        }
        print!("\n")
    }
}

// 打印网络翻译结果
fn print_web_tanslation_content(xml_node_list: &DictResult) {
    if let Some(trans) = &xml_node_list.web_translation {
        print_section_title("网络翻译");
        let mut print_next_line = false;
        for item in trans.iter() {
            if !print_next_line {
                print_next_line = true;
            } else {
                print!("\n");
            }
            print!("{} \n  ", style(&item.key).magenta());
            // print!("({})", value);
            item.value.iter().for_each(|v| {
                print!("{}; ", v);
            })
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
fn print_wiki(xml_node_list: &DictResult) {
    if let Some(wiki) = &xml_node_list.wiki {
        print_section_title("Wiki");
        println!("{}", style(format!("[ {} ]", wiki.entry)).green());

        println!("  {}", wiki.summary);
        print!("\n")
    }
}
