//pub mod deck;
//use deck::{Deck};

//fn main() {
    //let deck = Deck::standard();
    //println!("Deck has {} cards", deck.cards.len());
    //println!("First: {:?}, last: {:?}", deck.cards[0], deck.cards[51]);
//}
//

#[macro_use] extern crate rocket; 

#[get("/")] 
fn index() -> &'static str { 
    "Mein fuhrer, I can walk!" 
}

#[launch] 
fn rocket() -> { 
    rocket::build().mount("/", routes![index]); 
}
