#![no_std]
#![cfg_attr(test, no_main)]
#![test_runner(util::testing::unit_tests_runner)]
#![reexport_test_harness_main = "tests_main"]

#![feature(custom_test_frameworks)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(linkage)]

pub mod util;
pub mod early;
pub mod sync;
pub mod regapi;
pub mod kprint;
pub mod exceptions;

mod test_main;
