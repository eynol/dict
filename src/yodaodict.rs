use std::{borrow::Cow, fmt::Debug};

use console::{style, Attribute};
use quick_xml::de::from_str;
use serde::Deserialize;
use url::Url;

use crate::Args;
const  TRANSLATE_URL:&str = "http://dict.youdao.com/fsearch?version=1.1&client=deskdict&keyfrom=chrome.extension&doctype=xml&xmlVersion=3.2";

struct YouDaoTranslator<'a> {
    args: &'a Args,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct YouDaoDict<'a> {
    return_phrase: Option<Cow<'a, str>>,

    lang: Option<Cow<'a, str>>,

    phonetic_symbol: Option<Cow<'a, str>>,

    custom_translation: Option<Translation<'a>>,

    yodao_web_dict: Option<WebDict<'a>>,

    recommend: Option<Vec<Cow<'a, str>>>,

    wand_result: Option<WandResult<'a>>,
}

impl<'a> YouDaoDict<'a> {
    // 打印基础信息
    fn print_basic_info(&self) {
        if let Some(value) = &self.return_phrase {
            print!("  {}", style(value).bold().green());
        }

        if let Some(value) = &self.phonetic_symbol {
            print!("{}", style(format!("  [{}]", value)).bold().green().dim());
        }
        println!()
    }

    // 打印翻译结果
    fn print_tanslation_content(&self) {
        if let Some(list) = &self.custom_translation {
            print_section_title("翻译", true);
            list.translation
                .iter()
                .flat_map(|v| &v.content)
                .for_each(|value| {
                    println!("  {}", value);
                });
            println!()
        }
    }

    // 打印网络翻译结果
    fn print_web_tanslation_content(&self) {
        if let Some(webdict) = &self.yodao_web_dict {
            print_section_title("网络翻译", false);
            webdict.web_translation.iter().for_each(|translation| {
                print!("\n{} \n  ", style(&translation.key.trim()).magenta());

                let str1 = translation
                    .trans
                    .iter()
                    .map(|trans| {
                        format!(
                            "{}{}",
                            &trans.value.trim(),
                            &trans.cls.as_ref().map_or_else(|| { "" }, |x| { &x.cl })
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(";");

                print!("{}", str1)
            });
            println!()
        }
    }

    // 打印 Wiki 结果
    fn print_wiki(&self) {
        if let Some(wand) = &self.wand_result {
            print_section_title("Wiki", true);

            let entity = &wand.wand.entity;
            if let Some(wiki) = &entity.wiki_entry {
                println!("{}", style(format!("[ {} ]", wiki.info.data.entry)).green());
                println!("  {}", wiki.info.data.summary);
            }
            if let Some(person) = &entity.person_data {
                if person.name.is_some() {
                    println!(
                        "{}",
                        style(format!("[ {} ]", person.name.as_ref().unwrap())).green()
                    );
                    println!("  {}", person.nationality.as_ref().unwrap());
                    println!("  {}", person.summary.as_ref().unwrap());
                }
            }
            println!()
        }
    }
    fn print_to_stdio(&self) {
        self.print_basic_info();
        self.print_tanslation_content();
        self.print_web_tanslation_content();
        self.print_wiki();
    }
}

#[derive(Deserialize, Debug)]
struct Translation<'a> {
    translation: Vec<CustomTranslationContent<'a>>,
}
#[derive(Deserialize, Debug)]
struct CustomTranslationContent<'a> {
    content: Vec<Cow<'a, str>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct WebDict<'a> {
    web_translation: Vec<WebTranslation<'a>>,
}

#[derive(Deserialize, Debug)]
struct WebTranslation<'a> {
    key: Cow<'a, str>,

    trans: Vec<Value<'a>>,
}

#[derive(Deserialize, Debug)]
struct Value<'a> {
    value: Cow<'a, str>,
    cls: Option<CL<'a>>,
}

#[derive(Deserialize, Debug)]
struct CL<'a> {
    cl: Cow<'a, str>,
}

#[derive(Deserialize, Debug)]
struct WandResult<'a> {
    wand: Wand<'a>,
}
#[derive(Deserialize, Debug)]
struct Wand<'a> {
    entity: Entity<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Entity<'a> {
    wiki_entry: Option<WikiEntry<'a>>,
    person_data: Option<PersonData<'a>>,
    artist: Option<Artist<'a>>,
}

#[derive(Deserialize, Debug)]
struct WikiEntry<'a> {
    info: WikiInfo<'a>,
}

#[derive(Deserialize, Debug)]
struct WikiInfo<'a> {
    data: WikiInfoData<'a>,
}

#[derive(Deserialize, Debug)]
struct WikiInfoData<'a> {
    entry: Cow<'a, str>,
    summary: Cow<'a, str>,
}

#[derive(Deserialize, Debug)]
struct PersonData<'a> {
    name: Option<Cow<'a, str>>,
    summary: Option<Cow<'a, str>>,
    achievement: Option<Cow<'a, str>>,
    nationality: Option<Cow<'a, str>>,
    job: Option<Cow<'a, str>>,
    repWorks: Option<Cow<'a, str>>,
}

#[derive(Deserialize, Debug)]
struct Artist<'a> {
    name: Option<Cow<'a, str>>,
    brif_name: Option<Cow<'a, str>>,
}
trait Translator {
    fn translate_to_stdio(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

impl<'a> YouDaoTranslator<'a> {
    // 构建查询的url
    pub fn build_query_url(&self) -> String {
        let word = self.args.word.as_ref().unwrap();
        let mut parsed = Url::parse(TRANSLATE_URL).unwrap();
        parsed.query_pairs_mut().append_pair("q", word);
        let url = parsed.to_string();
        if self.args.debug {
            let url_to_debug = style(&url).attr(Attribute::Underlined);
            println!("url is {url_to_debug}",);
        }
        url
    }

    pub fn get_web_result_from_url(
        &self,
        url: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let resp = reqwest::blocking::get(url)?.text()?;
        if self.args.debug {
            println!(" xml data:{} \n", &resp)
        }
        Ok(resp)
    }
}
impl<'a> Translator for YouDaoTranslator<'a> {
    fn translate_to_stdio(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.build_query_url();
        let xml_str = self.get_web_result_from_url(url)?;

        let object: YouDaoDict = from_str(&xml_str)?;
        if self.args.debug {
            println!("result is {:?}", object);
        }

        object.print_to_stdio();
        Ok(())
    }
}

pub fn search_word(args: Args) {
    let mut youdao = YouDaoTranslator { args: &args };

    youdao.translate_to_stdio().unwrap();
}

pub fn print_section_title(title: &str, use_new_line: bool) {
    if use_new_line {
        println!(
            "========{txt:^width$}========",
            txt = style(title).magenta().dim(),
            width = 8
        );
    } else {
        print!(
            "========{txt:^width$}========",
            txt = style(title).magenta().dim(),
            width = 8
        );
    }
}
