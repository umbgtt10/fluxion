// Test what's available in futures-channel with alloc only
#![no_std]
extern crate alloc;

use futures_channel::oneshot;
// use futures_channel::mpsc; // Does this work?

fn test_oneshot() {
    let (_tx, _rx) = oneshot::channel::<i32>();
}

fn main() {
    test_oneshot();
}
