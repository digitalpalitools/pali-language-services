extern crate pali_language_services;

fn main() {
    println!("{:?}", pali_language_services::PALI_ALPHABET_ROMAN);
    let x = pali_language_services::PaliAlphabet::AA;
    println!("{:#?}", x > pali_language_services::PaliAlphabet::BH);
}
