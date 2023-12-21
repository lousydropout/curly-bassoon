#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod betting {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct Bettor {
        reviewers: Mapping<AccountId, ()>,
    }

    impl Bettor {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                reviewers: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn register_as_reviewer(&mut self) -> Result<(), ()> {
            self.reviewers.insert(self.env().caller(), &());
            Ok(())
        }

        #[ink(message)]
        pub fn is_registered_as_reviewer(&self) -> Result<bool, ()> {
            Ok(self.reviewers.contains(self.env().caller()))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<Environment>()
        }

        fn set_next_caller(caller: AccountId) {
            ink::env::test::set_caller::<Environment>(caller)
        }

        #[ink::test]
        fn register_alice() {
            set_next_caller(default_accounts().alice);
            let mut bettor = Bettor::default();

            // not registered yet
            assert_eq!(
                bettor.is_registered_as_reviewer(),
                Ok(false),
                "Alice was already registered"
            );

            let register = bettor.register_as_reviewer();
            assert!(register.is_ok(), "Unable to register Alice.");

            // registered
            assert_eq!(
                bettor.is_registered_as_reviewer(),
                Ok(true),
                "Alice is not registered"
            );
        }
    }
}
