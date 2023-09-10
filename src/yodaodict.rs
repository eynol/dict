use xml::reader::{EventReader, XmlEvent};
use url::Url;
const  TRANSLATE_URL:&str = "https://dict.youdao.com/fsearch?version=1.1&client=deskdict&keyfrom=chrome.extension&doctype=xml&xmlVersion=3.2";

// Language: rust
// 构建查询的url
pub fn build_query_url(word: &str) -> String {
    let mut parsed = Url::parse(TRANSLATE_URL).unwrap();
    parsed.query_pairs_mut().append_pair("q", word);
    parsed.to_string()
}
struct XmlData {
    cdata: String,
    character: String,
}
pub fn get_xml_node_list<'a>(xmldata: &'a str) -> Vec<(Vec<String>, String)> {
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
                    if let Some(key) = xml_paths.last() {
                        if key.eq("cl") {
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
