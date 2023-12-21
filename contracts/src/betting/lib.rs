#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod betting {
    use ink::prelude::string::String;

    #[ink(storage)]
    pub struct Greeter {
        message: String,
    }

    impl Greeter {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                message: "hi".into(),
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                message: "hi".into(),
            }
        }

        #[ink(message)]
        pub fn get_message(&self) -> Result<String, ()> {
            return Ok("hi".into());
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let x = Greeter::new();
            assert_eq!(x.get_message().unwrap(), String::from("hi"));
        }
    }
}
