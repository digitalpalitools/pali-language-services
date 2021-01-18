extern crate corelib;

fn main() {
    println!("{:?}", corelib::PALI_ALPHABET_ROMAN);
    let x = corelib::PaliAlphabet::AA;
    println!("{:#?}", x > corelib::PaliAlphabet::BH);
}
