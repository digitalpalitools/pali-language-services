// Spec: https://docs.google.com/document/d/1KF6NLFiiVH9oVz_NcU5mjHcMcIAZECgNifM8mX25MCo/edit#heading=h.es0rmyc509r7
pub const PALI_ALPHABET: &[&str] = &[
    "a", "ā", "i", "ī", "u", "ū", "e", "o", // vowels
    "k", "kh", "g", "gh", "ṅ", // guttural
    "c", "ch", "j", "jh", "ñ", // palatal
    "ṭ", "ṭh", "ḍ", "ḍh", "ṇ", // retroflex cerebral
    "t", "th", "d", "dh", "n", // dental
    "p", "ph", "b", "bh", "m", // labial
    "y", "r", "l", "v", "s", "h", "ḷ", // semi-vowel
    "ṃ", // nigahita
];

#[cfg(test)]
mod tests {
    use super::PALI_ALPHABET;

    #[test]
    fn test_pali_alphabet_length() {
        assert_eq!(PALI_ALPHABET.len(), 41);
    }
}
