use core::str::Chars;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use std::{convert::TryFrom, iter::Peekable};

// TODO: Split this into a Pāli core and a Roman specific module.

// Spec: https://docs.google.com/document/d/1KF6NLFiiVH9oVz_NcU5mjHcMcIAZECgNifM8mX25MCo/edit#heading=h.2hvqs8bpra4
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, IntoPrimitive, TryFromPrimitive)]
#[repr(usize)]
pub enum PaliAlphabet {
    A,
    AA,
    I,
    II,
    U,
    UU,
    E,
    O, // vowels - 0-7
    K,
    KH,
    G,
    GH,
    QuoteN, // guttural - 8-12
    C,
    CH,
    J,
    JH,
    TildeN, // palatal - 13-17
    DotT,
    DotTH,
    DotD,
    DotDH,
    DotN, // retroflex cerebral - 18-22
    T,
    TH,
    D,
    DH,
    N, // dental - 23-27
    P,
    PH,
    B,
    BH,
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
    if str1.len() != str2.len() {
        return isize::try_from(str1.len()).unwrap() - isize::try_from(str2.len()).unwrap();
    }

    let chars1 = CharacterTokenizer::new(str1.chars());
    let chars2 = CharacterTokenizer::new(str2.chars());

    let cmp = chars1
        .zip(chars2)
        .map(|(c1, c2)| char_compare(c1, c2))
        .find(|&sn| sn != 0);

    cmp.unwrap_or_default()
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
) -> Option<Character> {
    match chars.peek() {
        Some('h') => {
            chars.next();
            Some(Character::Pali(a2))
        }
        _ => Some(Character::Pali(a1)),
    }
}

fn parse_singlechar_letter(a: PaliAlphabet) -> Option<Character> {
    Some(Character::Pali(a))
}

impl<'a> Iterator for CharacterTokenizer<'a> {
    type Item = Character;

    fn next(&mut self) -> Option<Character> {
        match self.source.next() {
            Some('a') => parse_singlechar_letter(PaliAlphabet::A),
            Some('ā') => parse_singlechar_letter(PaliAlphabet::AA),
            Some('i') => parse_singlechar_letter(PaliAlphabet::I),
            Some('ī') => parse_singlechar_letter(PaliAlphabet::II),
            Some('u') => parse_singlechar_letter(PaliAlphabet::U),
            Some('ū') => parse_singlechar_letter(PaliAlphabet::UU),
            Some('e') => parse_singlechar_letter(PaliAlphabet::E),
            Some('o') => parse_singlechar_letter(PaliAlphabet::O),
            Some('k') => {
                parse_multichar_letter(&mut self.source, PaliAlphabet::K, PaliAlphabet::KH)
            }
            Some('g') => {
                parse_multichar_letter(&mut self.source, PaliAlphabet::G, PaliAlphabet::GH)
            }
            Some('ṅ') => parse_singlechar_letter(PaliAlphabet::QuoteN),
            Some('c') => {
                parse_multichar_letter(&mut self.source, PaliAlphabet::C, PaliAlphabet::CH)
            }
            Some('j') => {
                parse_multichar_letter(&mut self.source, PaliAlphabet::J, PaliAlphabet::JH)
            }
            Some('ñ') => parse_singlechar_letter(PaliAlphabet::TildeN),
            Some('ṭ') => {
                parse_multichar_letter(&mut self.source, PaliAlphabet::DotT, PaliAlphabet::DotTH)
            }
            Some('ḍ') => {
                parse_multichar_letter(&mut self.source, PaliAlphabet::DotD, PaliAlphabet::DotDH)
            }
            Some('ṇ') => parse_singlechar_letter(PaliAlphabet::DotN),
            Some('t') => {
                parse_multichar_letter(&mut self.source, PaliAlphabet::T, PaliAlphabet::TH)
            }
            Some('d') => {
                parse_multichar_letter(&mut self.source, PaliAlphabet::D, PaliAlphabet::DH)
            }
            Some('n') => parse_singlechar_letter(PaliAlphabet::N),
            Some('p') => {
                parse_multichar_letter(&mut self.source, PaliAlphabet::P, PaliAlphabet::PH)
            }
            Some('b') => {
                parse_multichar_letter(&mut self.source, PaliAlphabet::B, PaliAlphabet::BH)
            }
            Some('m') => parse_singlechar_letter(PaliAlphabet::M),
            Some('y') => parse_singlechar_letter(PaliAlphabet::Y),
            Some('r') => parse_singlechar_letter(PaliAlphabet::R),
            Some('l') => parse_singlechar_letter(PaliAlphabet::L),
            Some('v') => parse_singlechar_letter(PaliAlphabet::V),
            Some('s') => parse_singlechar_letter(PaliAlphabet::S),
            Some('h') => parse_singlechar_letter(PaliAlphabet::H),
            Some('ḷ') => parse_singlechar_letter(PaliAlphabet::DotL),
            Some('ṃ') => parse_singlechar_letter(PaliAlphabet::DotM),
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

    #[test_case("c", "cc", -1)]
    #[test_case("c", "b", -1)]
    #[test_case("c", "c", 0)]
    #[test_case("b", "c", 1)]
    #[test_case("cc", "c", 1)]
    #[test_case("ac", "ab", -1)]
    #[test_case("ac", "ac", 0)]
    #[test_case("ab", "ac", 1)]
    #[test_case("a", "x", -1)]
    #[test_case("x", "a", 1)]
    #[test_case("x", "z", -1)]
    #[test_case("x", "x", 0)]
    #[test_case("z", "x", 1)]
    #[test_case("xabc", "aabc", 1)]
    #[test_case("aabc", "xabc", -1)]
    #[test_case("xabc", "yabc", 1)]
    #[test_case("xabc", "xabc", 0)]
    #[test_case("yabc", "xabc", -1)]
    fn multiplication_tests(str1: &str, str2: &str, cmp: isize) {
        let cmp_actual = string_compare(str1, str2);

        assert_eq!(cmp_actual, cmp)
    }

    proptest! {
        #[test]
        fn fixup_compound_letters_with_compound_letters(index in 0usize..PALI_ALPHABET_ROMAN_COMPOUND_LETTERS_INDICES.len()) {
            let indices: Vec<usize> = vec![0, PALI_ALPHABET_ROMAN_COMPOUND_LETTERS_INDICES[index], 38, 38, 2, 38];
            let fixed_indices = fixup_compound_letters(&indices);

            let new_indices = vec![0, PALI_ALPHABET_ROMAN_COMPOUND_LETTERS_INDICES[index] + 1, 38, 2, 38];

            assert_eq!(new_indices, fixed_indices)
        }

        #[test]
        fn round_trip_pali_to_roman(index in 0usize..PALI_ALPHABET_ROMAN.len()) {
            let pali_char = PaliAlphabet::try_from(index).unwrap();
            let i: usize = pali_char.into();
            let str = PALI_ALPHABET_ROMAN[i];

            let tokenizer = CharacterTokenizer::new(str.chars());
            let new_pali_char = tokenizer.map(|c| match c { Character::Pali(c) => c, _ => panic!("") }).next().unwrap();

            assert_eq!(new_pali_char, pali_char);
        }

        #[test]
        fn round_trip_parsing_for_long_strings(indices in prop::collection::vec(0usize..PALI_ALPHABET_ROMAN.len(), 0..100)) {
            let indices = fixup_compound_letters(&indices);

            let pali_string = indices
                .iter()
                .map(|&i| PALI_ALPHABET_ROMAN[i] )
                .fold(String::new(), |acc, e| { acc + e });

            let tokenizer = CharacterTokenizer::new(pali_string.chars());
            let new_indices: Vec<usize> = tokenizer.map(|c| match c { Character::Pali(c) => c.into(), _ => panic!("") }).collect();

            assert_eq!(new_indices, indices);
        }
    }
}
