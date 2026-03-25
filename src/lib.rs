#![doc = include_str!("../README.md")]
#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn create_and_delete_project() {
        unsafe {
            let mut ph: EN_Project = ptr::null_mut();
            assert_eq!(EN_createproject(&mut ph), 0);
            assert_eq!(EN_deleteproject(ph), 0);
        }
    }
}