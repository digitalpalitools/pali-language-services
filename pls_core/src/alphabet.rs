use core::str::Chars;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{convert::TryFrom, iter::Peekable};

// TODO: Split this into a Pāli core and a Roman specific module.

// Spec: https://docs.google.com/document/d/1KF6NLFiiVH9oVz_NcU5mjHcMcIAZECgNifM8mX25MCo/edit#heading=h.2hvqs8bpra4
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, IntoPrimitive, TryFromPrimitive)]
#[repr(usize)]
pub enum PaliAlphabet {
    A,
    Aa,
    I,
    Ii,
    U,
    Uu,
    E,
    O, // vowels - 0-7
    K,
    Kh,
    G,
    Gh,
    QuoteN, // guttural - 8-12
    C,
    Ch,
    J,
    Jh,
    TildeN, // palatal - 13-17
    DotT,
    DotTH,
    DotD,
    DotDH,
    DotN, // retroflex cerebral - 18-22
    T,
    Th,
    D,
    Dh,
    N, // dental - 23-27
    P,
    Ph,
    B,
    Bh,
    M, // labial - 28-32
    Y,
    R,
    L,
    V,
    S,
    H,
    DotL, // semi-vowel - 33-39
    DotM, // nigahita - 40-40
}

pub const PALI_ALPHABET_ROMAN: &[&str] = &[
    "a", "ā", "i", "ī", "u", "ū", "e", "o", // vowels - 0-7
    "k", "kh", "g", "gh", "ṅ", // guttural - 8-12
    "c", "ch", "j", "jh", "ñ", // palatal - 13-17
    "ṭ", "ṭh", "ḍ", "ḍh", "ṇ", // retroflex cerebral - 18-22
    "t", "th", "d", "dh", "n", // dental - 23-27
    "p", "ph", "b", "bh", "m", // labial - 28-32
    "y", "r", "l", "v", "s", "h", "ḷ", // semi-vowel - 33-39
    "ṃ", // nigahita - 40-40
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Character {
    Pali(PaliAlphabet),
    Other(char),
}

pub struct CharacterTokenizer<'a> {
    source: Peekable<Chars<'a>>,
}

pub fn char_compare(c1: Character, c2: Character) -> isize {
    match (c1, c2) {
        (Character::Other(c1), Character::Other(c2)) => (c1 as isize - c2 as isize).signum(),
        (Character::Pali(c1), Character::Pali(c2)) => (c1 as isize - c2 as isize).signum(),
        (Character::Other(_c1), Character::Pali(_c2)) => 1,
        (Character::Pali(_c1), Character::Other(_c2)) => -1,
    }
}

pub fn string_compare(str1: &str, str2: &str) -> isize {
    let chars1 = CharacterTokenizer::new(str1.chars());
    let chars2 = CharacterTokenizer::new(str2.chars());

    let cmp = chars1
        .zip(chars2)
        .map(|(c1, c2)| char_compare(c1, c2))
        .find(|&sn| sn != 0);

    match cmp {
        Some(cmp) => cmp,
        None => {
            // TODO: This is a temp hack. Tokenize just once.
            let str1len = string_length(str1);
            let str2len = string_length(str2);
            if str1len != str2len {
                (isize::try_from(str1len).unwrap() - isize::try_from(str2len).unwrap()).signum()
            } else {
                0isize
            }
        }
    }
}

pub fn string_length(str1: &str) -> usize {
    let chars1 = CharacterTokenizer::new(str1.chars());

    chars1.count()
}

impl<'a> CharacterTokenizer<'a> {
    pub fn new(source: Chars<'a>) -> CharacterTokenizer<'a> {
        CharacterTokenizer {
            source: source.peekable(),
        }
    }
}

fn parse_multichar_letter(
    chars: &mut Peekable<Chars<'_>>,
    a1: PaliAlphabet,
    a2: PaliAlphabet,
) -> Character {
    match chars.peek() {
        Some('h') => {
            chars.next();
            Character::Pali(a2)
        }
        _ => Character::Pali(a1),
    }
}

fn parse_singlechar_letter(a: PaliAlphabet) -> Character {
    Character::Pali(a)
}

impl<'a> Iterator for CharacterTokenizer<'a> {
    type Item = Character;

    fn next(&mut self) -> Option<Character> {
        match self.source.next() {
            Some('a') => Some(parse_singlechar_letter(PaliAlphabet::A)),
            Some('ā') => Some(parse_singlechar_letter(PaliAlphabet::AA)),
            Some('i') => Some(parse_singlechar_letter(PaliAlphabet::I)),
            Some('ī') => Some(parse_singlechar_letter(PaliAlphabet::II)),
            Some('u') => Some(parse_singlechar_letter(PaliAlphabet::U)),
            Some('ū') => Some(parse_singlechar_letter(PaliAlphabet::UU)),
            Some('e') => Some(parse_singlechar_letter(PaliAlphabet::E)),
            Some('o') => Some(parse_singlechar_letter(PaliAlphabet::O)),
            Some('k') => Some(parse_multichar_letter(
                &mut self.source,
                PaliAlphabet::K,
                PaliAlphabet::KH,
            )),
            Some('g') => Some(parse_multichar_letter(
                &mut self.source,
                PaliAlphabet::G,
                PaliAlphabet::GH,
            )),
            Some('ṅ') => Some(parse_singlechar_letter(PaliAlphabet::QuoteN)),
            Some('c') => Some(parse_multichar_letter(
                &mut self.source,
                PaliAlphabet::C,
                PaliAlphabet::CH,
            )),
            Some('j') => Some(parse_multichar_letter(
                &mut self.source,
                PaliAlphabet::J,
                PaliAlphabet::JH,
            )),
            Some('ñ') => Some(parse_singlechar_letter(PaliAlphabet::TildeN)),
            Some('ṭ') => Some(parse_multichar_letter(
                &mut self.source,
                PaliAlphabet::DotT,
                PaliAlphabet::DotTH,
            )),
            Some('ḍ') => Some(parse_multichar_letter(
                &mut self.source,
                PaliAlphabet::DotD,
                PaliAlphabet::DotDH,
            )),
            Some('ṇ') => Some(parse_singlechar_letter(PaliAlphabet::DotN)),
            Some('t') => Some(parse_multichar_letter(
                &mut self.source,
                PaliAlphabet::T,
                PaliAlphabet::TH,
            )),
            Some('d') => Some(parse_multichar_letter(
                &mut self.source,
                PaliAlphabet::D,
                PaliAlphabet::DH,
            )),
            Some('n') => Some(parse_singlechar_letter(PaliAlphabet::N)),
            Some('p') => Some(parse_multichar_letter(
                &mut self.source,
                PaliAlphabet::P,
                PaliAlphabet::PH,
            )),
            Some('b') => Some(parse_multichar_letter(
                &mut self.source,
                PaliAlphabet::B,
                PaliAlphabet::BH,
            )),
            Some('m') => Some(parse_singlechar_letter(PaliAlphabet::M)),
            Some('y') => Some(parse_singlechar_letter(PaliAlphabet::Y)),
            Some('r') => Some(parse_singlechar_letter(PaliAlphabet::R)),
            Some('l') => Some(parse_singlechar_letter(PaliAlphabet::L)),
            Some('v') => Some(parse_singlechar_letter(PaliAlphabet::V)),
            Some('s') => Some(parse_singlechar_letter(PaliAlphabet::S)),
            Some('h') => Some(parse_singlechar_letter(PaliAlphabet::H)),
            Some('ḷ') => Some(parse_singlechar_letter(PaliAlphabet::DotL)),
            Some('ṃ') => Some(parse_singlechar_letter(PaliAlphabet::DotM)),
            Some(c) => Some(Character::Other(c)),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::convert::TryFrom;
    use test_case::test_case;

    #[test]
    fn test_pali_alphabet_length() {
        assert_eq!(PALI_ALPHABET_ROMAN.len(), 41);

        let i: usize = PaliAlphabet::DotM.into();
        assert_eq!(i, 40);
    }

    const PALI_ALPHABET_ROMAN_COMPOUND_LETTERS_INDICES: &[usize] =
        &[8, 10, 13, 15, 18, 20, 23, 25, 28, 30];

    const PALI_ALPHABET_ROMAN_COMPOUNDING_LETTER_INDEX: usize = 38;

    fn is_compound_letter_roman(index: usize) -> bool {
        None != PALI_ALPHABET_ROMAN_COMPOUND_LETTERS_INDICES
            .iter()
            .find(|&&e| e == index)
    }

    fn fixup_compound_letters(indices: &[usize]) -> Vec<usize> {
        let ret = indices
            .iter()
            .fold((None, Vec::new()), |(ep, mut acc), &e| {
                match ep {
                    Some(ep) => {
                        if e == PALI_ALPHABET_ROMAN_COMPOUNDING_LETTER_INDEX
                            && is_compound_letter_roman(ep)
                        {
                            let i_prev = acc.len() - 1;
                            acc[i_prev] += 1;
                        } else {
                            acc.push(e);
                        }
                    }
                    None => acc.push(e),
                }
                (Some(e), acc)
            });

        ret.1
    }

    #[test]
    fn parse_with_non_pali_character() {
        let str = "xā1b";
        let tokenizer = CharacterTokenizer::new(str.chars());

        let chars: Vec<_> = tokenizer.collect();

        assert_eq!(
            chars,
            vec![
                Character::Other('x'),
                Character::Pali(PaliAlphabet::AA),
                Character::Other('1'),
                Character::Pali(PaliAlphabet::B)
            ]
        )
    }

    #[test]
    fn fixup_compound_letters_with_no_compound_letters() {
        let indices: Vec<usize> = vec![0, 1, 2];
        let fixedup_indices = fixup_compound_letters(&indices);

        assert_eq!(fixedup_indices, indices)
    }

    #[test_case("c", "cc"   => -1)]
    #[test_case("c", "b"    => -1)]
    #[test_case("c", "c"    => 0)]
    #[test_case("b", "c"    => 1)]
    #[test_case("cc", "c"   => 1)]
    #[test_case("ac", "ab"  => -1)]
    #[test_case("ac", "ac"  => 0)]
    #[test_case("ab", "ac"  => 1)]
    #[test_case("a", "x"    => -1)]
    #[test_case("x", "a"    => 1)]
    #[test_case("x", "z"    => -1)]
    #[test_case("x", "x"    => 0)]
    #[test_case("z", "x"    => 1)]
    #[test_case("xabc", "aabc"  => 1)]
    #[test_case("aabc", "xabc"  => -1)]
    #[test_case("xabc", "yabc"  => 1)]
    #[test_case("xabc", "xabc"  => 0)]
    #[test_case("yabc", "xabc"  => -1)]
    #[test_case("i", "ā"    => 1; "random letters 1")]
    #[test_case("cc", "b"   => -1; "longer of lesser sort order 1")]
    fn string_compare_tests(str1: &str, str2: &str) -> isize {
        string_compare(str1, str2)
    }

    #[test_case("buddho" => 5usize; "simple word 1")]
    #[test_case("bhagavā" => 6usize; "simple word 2")]
    #[test_case("aāiīuūeokkhgghṅcchjjhñṭṭhḍḍhṇtthddhnpphbbhmyrlvshḷṃ" => 41; "all characters")]
    fn string_length_tests(str1: &str) -> usize {
        string_length(str1)
    }

    proptest! {
        #[test]
        fn string_compare_all(i1 in 0usize..PALI_ALPHABET_ROMAN.len(), i2 in 0usize..PALI_ALPHABET_ROMAN.len()) {
            let cmp_str = string_compare(PALI_ALPHABET_ROMAN[i1], PALI_ALPHABET_ROMAN[i2]);

            let pali_char1 = PaliAlphabet::try_from(i1).expect("Catastrophic unhandlable error.");
            let pali_char2 = PaliAlphabet::try_from(i2).expect("Catastrophic unhandlable error.");
            let cmp_char = (pali_char1 as isize - pali_char2 as isize).signum();

            assert_eq!(cmp_char, cmp_str);
        }

        #[test]
        fn fixup_compound_letters_with_compound_letters(index in 0usize..PALI_ALPHABET_ROMAN_COMPOUND_LETTERS_INDICES.len()) {
            let indices: Vec<usize> = vec![0, PALI_ALPHABET_ROMAN_COMPOUND_LETTERS_INDICES[index], 38, 38, 2, 38];
            let fixed_indices = fixup_compound_letters(&indices);

            let new_indices = vec![0, PALI_ALPHABET_ROMAN_COMPOUND_LETTERS_INDICES[index] + 1, 38, 2, 38];

            assert_eq!(new_indices, fixed_indices)
        }

        #[test]
        fn round_trip_pali_to_roman(index in 0usize..PALI_ALPHABET_ROMAN.len()) {
            let pali_char = PaliAlphabet::try_from(index).expect("Catastrophic unhandlable error.");
            let i: usize = pali_char.into();
            let str = PALI_ALPHABET_ROMAN[i];

            let tokenizer = CharacterTokenizer::new(str.chars());
            let new_pali_char = tokenizer.map(|c| match c { Character::Pali(c) => c, _ => panic!("") }).next().expect("Catastrophic unhandlable error.");

            assert_eq!(new_pali_char, pali_char);
        }

        #[test]
        fn round_trip_parsing_for_long_strings(indices in prop::collection::vec(0usize..PALI_ALPHABET_ROMAN.len(), 0..100)) {
            let indices = fixup_compound_letters(&indices);

            let pali_string = indices
                .iter()
                .map(|&i| PALI_ALPHABET_ROMAN[i] )
                .fold(String::new(), |acc, e| acc + e);

            let tokenizer = CharacterTokenizer::new(pali_string.chars());
            let new_indices: Vec<usize> = tokenizer.map(|c| match c { Character::Pali(c) => c.into(), _ => panic!("") }).collect();

            assert_eq!(new_indices, indices);
        }
    }
}
