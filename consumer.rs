#![feature(phase)]

#[phase(plugin)]
extern crate rangedtype;

use std::num::FromPrimitive;

ranged_type!(Digit, 0, 9)

ranged_type!(SizeTest, 0, 1023)

fn main() {
	let x = Digit3;
	let y: Digit = FromPrimitive::from_int(9).unwrap();
	println!("{} and {}", Digit2+Digit2, y-x);
}