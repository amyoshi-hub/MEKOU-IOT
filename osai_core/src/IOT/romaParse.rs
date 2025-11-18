pub fn split_romaji(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    let two_syllables = [
    "a", "i", "u", "e", "o", // 母音

    // 清音
    "ka", "ki", "ku", "ke", "ko",
    "sa", "shi", "su", "se", "so",
    "ta", "chi", "tsu", "te", "to",
    "na", "ni", "nu", "ne", "no",
    "ha", "hi", "fu", "he", "ho",
    "ma", "mi", "mu", "me", "mo",
    "ya", "yu", "yo",
    "ra", "ri", "ru", "re", "ro",
    "wa",

    // 濁音
    "ga", "gi", "gu", "ge", "go",
    "za", "ji", "zu", "ze", "zo",
    "da", "de", "do",
    "ba", "bi", "bu", "be", "bo",

    // 半濁音
    "pa", "pi", "pu", "pe", "po",

    // 外来音
    "fa", "fi", "fe", "fo",
    "va", "vi", "vu", "ve", "vo",

    ];
    let three_syllables = [
    "kya", "kyu", "kyo",
    "sha", "shi", "shu",
    "she", "sho", "sha",//shaは見た目上の都合残す
    "cha", "chu", "cho",
    "nya", "nyu", "nyo",
    "hya", "hyu", "hyo",
    "mya", "myu", "myo",
    "rya", "ryu", "ryo",
    "gya", "gyu", "gyo",
    "bya", "byu", "byo",
    "pya", "pyu", "pyo",
    "ja",  "ju",  "jo",
    "tya", "tyu", "tyo", // ti系対応
    "chi", "cho", "cha",
    "dya", "dyu", "dyo", // di系対応
    "sya", "syu", "syo", // si系対応
    ];

    while i < chars.len() {
        let rest = &chars[i..];

        // 3文字音節
        if rest.len() >= 3 {
            let tri = rest[0..3].iter().collect::<String>();
            if three_syllables.contains(&tri.as_str()) {
                result.push(tri);
                i += 3;
                continue;
            }
        }

        // 2文字音節
        if rest.len() >= 2 {
            let bi = rest[0..2].iter().collect::<String>();
            if two_syllables.contains(&bi.as_str()) {
                result.push(bi);
                i += 2;
                continue;
            }
        }

        // 1文字音節（n など）
        result.push(chars[i].to_string());
        i += 1;
    }

    result
}
