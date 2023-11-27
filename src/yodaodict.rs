use std::path::PathBuf;

use url::Url;
use xml::{
    reader::{EventReader, XmlEvent},
    ParserConfig,
};
const  TRANSLATE_URL:&str = "https://dict.youdao.com/fsearch?version=1.1&client=deskdict&keyfrom=chrome.extension&doctype=xml&xmlVersion=3.2";

// Language: rust
// 构建查询的url
pub fn build_query_url(word: &str) -> String {
    let mut parsed = Url::parse(TRANSLATE_URL).unwrap();
    parsed.query_pairs_mut().append_pair("q", word);
    parsed.to_string()
}
pub struct XmlData {
    cdata: String,
    character: String,
}

#[derive(Debug)]
pub struct DictResult {
    pub query: String,
    pub return_phrase: Option<String>,
    pub lang: String,
    pub phonetic_symbol: Option<String>,
    pub translation: Option<CustomTranslation>,
    pub web_translation: Option<Vec<WebTranslation>>,
    pub wiki: Option<Wiki>,
}
#[derive(Debug)]
pub struct CustomTranslation {
    pub kind: String,
    pub content: Vec<String>,
}
#[derive(Debug)]
pub struct WebTranslation {
    pub key: String,
    pub value: Vec<String>,
}

#[derive(Debug)]
pub struct Wiki {
    pub entry: String,
    pub summary: String,
}

static EMPTY_STR: &str = "";
pub fn get_xml_node_list<'a>(word: &String, xmldata: &'a str) -> DictResult {
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
