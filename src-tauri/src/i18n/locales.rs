use serde::{Serialize, Deserialize};

/// 应用支持的语言 / 区域（基于目标客户群）
#[derive(Debug, Serialize, Deserialize)]
pub struct Locales {

    // ---- 东亚 ----
    #[serde(rename = "zh-CN")] // 简体中文（中国大陆）
    pub zh_cn: bool,

    #[serde(rename = "zh-TW")] // 繁体中文（台湾 / 港澳）
    pub zh_tw: bool,

    #[serde(rename = "ja-JP")] // 日语
    pub ja_jp: bool,

    #[serde(rename = "ko-KR")] // 韩语
    pub ko_kr: bool,


    // ---- 东南亚 ----
    #[serde(rename = "en-SG")] // 新加坡英语
    pub en_sg: bool,

    #[serde(rename = "vi-VN")] // 越南语
    pub vi_vn: bool,

    #[serde(rename = "ms-MY")] // 马来语（马来西亚）
    pub ms_my: bool,

    #[serde(rename = "id-ID")] // 印度尼西亚语
    pub id_id: bool,


    // ---- 南亚 ----
    #[serde(rename = "en-IN")] // 印度英语（最广用）
    pub en_in: bool,

    #[serde(rename = "hi-IN")] // 印地语
    pub hi_in: bool,


    // ---- 南美 ----
    #[serde(rename = "es-AR")] // 阿根廷西班牙语
    pub es_ar: bool,

    #[serde(rename = "es-VE")] // 委内瑞拉西班牙语
    pub es_ve: bool,

    #[serde(rename = "pt-BR")] // 巴西葡萄牙语
    pub pt_br: bool,


    // ---- 欧美基本语言（不是重点但需要） ----
    #[serde(rename = "en-US")] // 英语（美国）
    pub en_us: bool,

    #[serde(rename = "es-ES")] // 西班牙语（欧洲）
    pub es_es: bool,

    #[serde(rename = "fr-FR")] // 法语
    pub fr_fr: bool,

    #[serde(rename = "de-DE")] // 德语
    pub de_de: bool,

    #[serde(rename = "ru-RU")] // 俄语
    pub ru_ru: bool,
}

impl Default for Locales {
    fn default() -> Self {
        Self {
            zh_cn: true,
            zh_tw: true,
            ja_jp: true,
            ko_kr: true,
            en_sg: true,
            vi_vn: true,
            ms_my: true,
            id_id: true,
            en_in: true,
            hi_in: true,
            es_ar: true,
            es_ve: true,
            pt_br: true,
            en_us: true,
            es_es: true,
            fr_fr: true,
            de_de: true,
            ru_ru: true,
        }
    }
}
