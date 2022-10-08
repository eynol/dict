extern crate xml;

use console::{style, Attribute};
use std::env;
use url::Url;
use xml::reader::{EventReader, XmlEvent};

struct CliOptions {
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let tt = vec![1, 2, 3, 4, 5];
    // test(tt);

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <单词>", args[0]);
        return Ok(());
    }

    let options = CliOptions {
        debug: args.contains(&"--debug".to_string()),
    };

    let word = &args[1];
    if word == "-v" || word == "--version" {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        println!("v{}", VERSION);
        return Ok(());
    }
    let url = build_query_url(word);
    if options.debug {
        println!("url: {}", style(&url).attr(Attribute::Underlined));
    }

    let resp = reqwest::blocking::get(url)?.text()?;
    let xml_node_list = get_xml_node_list(&resp);
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

// Language: rust
// 构建查询的url
fn build_query_url(word: &str) -> String {
    let url_1 = "https://dict.youdao.com/fsearch?version=1.1&client=deskdict&keyfrom=chrome.extension&doctype=xml&xmlVersion=3.2";
    let mut parsed = Url::parse(url_1).unwrap();
    parsed.query_pairs_mut().append_pair("q", word);
    parsed.to_string()
}

struct XmlData {
    cdata: String,
    character: String,
}

fn get_xml_node_list<'a>(xmldata: &'a str) -> Vec<(Vec<String>, String)> {
    let mut xml_node_list: Vec<(Vec<String>, String)> = Vec::new();

    let parser = EventReader::from_str(xmldata);
    let mut xml_paths: Vec<String> = Vec::new();

    let mut temp_data = XmlData {
        cdata: "".to_string(),
        character: "".to_string(),
    };

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                if !name.local_name.eq("yodaodict") {
                    xml_paths.push(name.local_name);
                }
            }
            Ok(XmlEvent::CData(data)) => {
                temp_data.cdata = data;
            }
            Ok(XmlEvent::Characters(data)) => {
                temp_data.character = data.trim().to_string();
            }
            Ok(XmlEvent::EndElement { name }) => {
                if temp_data.cdata.len() > 0 || temp_data.character.len() > 0 {
                    // 如果是计算机目录类别的，需要合并进前一个的里面
                    if xml_paths.ends_with(&["cl".to_string()]) {
                        let last_value = &mut xml_node_list.last_mut().unwrap().1;
                        last_value.push_str("(");
                        last_value.push_str(&temp_data.cdata);
                        last_value.push_str(&temp_data.character);
                        last_value.push_str(")");
                    } else {
                        xml_node_list.push((
                            xml_paths.clone(),
                            format!("{}{}", temp_data.cdata, temp_data.character),
                        ));
                    }
                }
                temp_data.cdata = "".to_string();
                temp_data.character = "".to_string();
                if !name.to_string().eq("yodaodict") {
                    xml_paths.pop();
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
    xml_node_list
}

fn get_key_value<'a>(
    xml_node_list: &'a Vec<(Vec<String>, String)>,
    key: &String,
) -> Option<&'a (Vec<String>, String)> {
    xml_node_list.iter().find(|x| x.0.get(0) == Some(key))
}

// fn test(arr: Vec<i32>) -> String {
//     // let arr = vec![12, 3, 4, 4, 7, 5, 56];
//     arr.iter().filter(|x| x > &5)
// }

fn filter_prefix<'a, 'b>(
    xml_node_list: &'a Vec<(Vec<String>, String)>,
    keys: &'b Vec<String>,
) -> Vec<&'a (Vec<String>, String)> {
    return xml_node_list
        .iter()
        .filter(|x| x.0.starts_with(keys))
        .collect();
}

// 打印基础信息
fn print_basic_info(xml_node_list: &Vec<(Vec<String>, String)>) {
    if let Some((_, value)) = get_key_value(xml_node_list, &"return-phrase".to_string()) {
        print!("  {}", style(value).bold().green());
    }

    if let Some((_, value)) = get_key_value(xml_node_list, &"phonetic-symbol".to_string()) {
        print!("{}", style(format!("  [{}]", value)).bold().green().dim());
    }
    print!("\n")
}

// 打印翻译结果
fn print_tanslation_content(xml_node_list: &Vec<(Vec<String>, String)>) {
    let list = filter_prefix(
        xml_node_list,
        &vec![
            "custom-translation".to_string(),
            "translation".to_string(),
            "content".to_string(),
        ],
    );

    if list.len() > 0 {
        print_section_title("翻译");
        for (_, value) in list.iter() {
            println!("  {}", value);
        }
        print!("\n")
    }
}

// 打印网络翻译结果
fn print_web_tanslation_content(xml_node_list: &Vec<(Vec<String>, String)>) {
    let list = filter_prefix(
        xml_node_list,
        &"yodao-web-dict web-translation"
            .split_ascii_whitespace()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    );

    if list.len() > 0 {
        print_section_title("网络翻译");
        let mut print_next_line = false;
        for (paths, value) in list.iter() {
            if paths.ends_with(&["key"].map(|x| x.to_string())) {
                if !print_next_line {
                    print_next_line = true;
                } else {
                    print!("\n");
                }
                print!("{} \n  ", style(value).magenta());
            } else if paths.ends_with(&["cl".to_string()]) {
                // print!("({})", value);
            } else {
                print!("{}; ", value);
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
    let list = filter_prefix(
        xml_node_list,
        &"wand-result wand entity wiki-entry info data"
            .split_ascii_whitespace()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
    );

    if list.len() > 0 {
        print_section_title("Wiki");
        for (paths, value) in list.iter() {
            if paths.ends_with(&["entry".to_string()]) {
                println!("{}", style(format!("[ {} ]", value)).green());
            } else if paths.ends_with(&["wikiType".to_string()]) {
                // println!("    {}", value);
            } else if paths.ends_with(&["summary".to_string()]) {
                println!("  {}", value);
            }
        }
        print!("\n")
    }
}
