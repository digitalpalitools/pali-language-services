extern crate pali_core;

fn main() {
    println!("{:?}", pali_core::PALI_ALPHABET_ROMAN);
    let x = pali_core::PaliAlphabet::AA;
    println!("{:#?}", x > pali_core::PaliAlphabet::BH);
}
