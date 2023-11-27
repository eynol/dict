use console::{style, Attribute};
mod yodaodict;
use std::{env, path::PathBuf};

use xml::{
    reader::{EventReader, XmlEvent},
    ParserConfig,
};

struct CliOptions {
    debug: bool,
}

static mut DEBUG_MODE: bool = false;
#[derive(Debug)]
struct DictResult {
    query: String,
    return_phrase: Option<String>,
    lang: String,
    phonetic_symbol: Option<String>,
    translation: Option<CustomTranslation>,
    web_translation: Option<Vec<WebTranslation>>,
    wiki: Option<Wiki>,
}
#[derive(Debug)]
struct CustomTranslation {
    kind: String,
    content: Vec<String>,
}
#[derive(Debug)]
struct WebTranslation {
    key: String,
    value: Vec<String>,
}

#[derive(Debug)]
struct Wiki {
    entry: String,
    summary: String,
}
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
    let dict_rs = get_xml_node_list(&word, &resp);
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

struct XmlData {
    cdata: String,
    character: String,
}

fn get_xml_node_list<'a>(word: &String, xmldata: &'a str) -> DictResult {
    let mut dict_rs = DictResult {
        query: word.clone(),
        return_phrase: None,
        lang: String::from("eng"),
        phonetic_symbol: None,
        translation: None,
        web_translation: None,
        wiki: None,
    };

    let empty_str = "".to_string();
    let config = ParserConfig {
        trim_whitespace: true,
        ..ParserConfig::default()
    };
    let parser = EventReader::new_with_config(xmldata.as_bytes(), config);

    let mut temp_data = XmlData {
        cdata: empty_str.clone(),
        character: empty_str.clone(),
    };
    let mut path_buf = PathBuf::new();

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                if name.local_name != "yodaodict" {
                    path_buf.push(name.local_name);
                }
            }
            Ok(XmlEvent::CData(data)) => {
                temp_data.cdata = data;
            }
            Ok(XmlEvent::Characters(data)) => {
                temp_data.character = data;
            }
            Ok(XmlEvent::EndElement { name }) => {
                if path_buf.ends_with("return-phrase") {
                    dict_rs.return_phrase = Some(temp_data.cdata + &temp_data.character);
                } else if path_buf.ends_with("custom-translation/type") {
                    let dict = dict_rs.translation.get_or_insert(CustomTranslation {
                        kind: empty_str.clone(),
                        content: vec![],
                    });
                    dict.kind = temp_data.character;
                } else if path_buf.ends_with("custom-translation/translation/content") {
                    if let Some(trans) = &mut dict_rs.translation {
                        trans.content.push(temp_data.cdata);
                    };
                } else if path_buf.ends_with("phonetic-symbol") {
                    dict_rs.phonetic_symbol = Some(temp_data.character)
                } else if path_buf.ends_with("web-translation/key") {
                    let list = dict_rs.web_translation.get_or_insert(Vec::new());
                    list.push(WebTranslation {
                        key: temp_data.cdata + &temp_data.character,
                        value: vec![],
                    });
                } else if path_buf.ends_with("web-translation/trans/value") {
                    if let Some(list) = dict_rs.web_translation.as_mut().unwrap().last_mut() {
                        list.value.push(temp_data.cdata + &temp_data.character);
                    };
                } else if path_buf.ends_with("web-translation/trans/value/cl") {
                    if temp_data.cdata.len() > 0 || temp_data.character.len() > 0 {
                        // 如果是计算机目录类别的，需要合并进前一个的value里面
                        if let Some(web_translation) =
                            dict_rs.web_translation.as_mut().unwrap().last_mut()
                        {
                            let last_value = web_translation.value.last_mut().unwrap();
                            last_value.push_str("(");
                            last_value.push_str(&temp_data.cdata);
                            last_value.push_str(&temp_data.character);
                            last_value.push_str(")");
                        };
                    }
                } else if path_buf.ends_with("wiki-entry/info/data/entry") {
                    dict_rs.wiki = Some(Wiki {
                        entry: temp_data.character,
                        summary: empty_str.clone(),
                    });
                } else if path_buf.ends_with("wiki-entry/info/data/summary") {
                    dict_rs.wiki.as_mut().unwrap().summary = temp_data.character;
                }

                // println!("{}", path_buf.display());
                temp_data.cdata = empty_str.clone();
                temp_data.character = empty_str.clone();
                if !name.to_string().eq("yodaodict") {
                    path_buf.pop();
                }
                // println!("{}-{}", indent(depth), name);
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    dict_rs
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
