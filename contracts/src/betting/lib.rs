#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod betting {
    use ink::storage::Mapping;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        // Caller is not a final decision maker
        NotFinalDecisionMaker,
    }

    #[ink(storage)]
    pub struct Betting {
        reviewers: Mapping<AccountId, ()>,
        number_of_reviewers: u32,
        final_decision_maker: AccountId,
    }

    impl Betting {
        #[ink(constructor)]
        pub fn new(final_decision_maker: AccountId) -> Self {
            Self {
                reviewers: Mapping::default(),
                number_of_reviewers: 0,
                final_decision_maker,
            }
        }

        // --------------------------------------------------------
        // Reputation-related functions
        // --------------------------------------------------------
        #[ink(message)]
        pub fn register_as_reviewer(&mut self) -> Result<(), ()> {
            self.reviewers.insert(self.env().caller(), &());
            self.number_of_reviewers += 1;
            Ok(())
        }

        #[ink(message)]
        pub fn is_registered_as_reviewer(&self) -> Result<bool, ()> {
            Ok(self.reviewers.contains(self.env().caller()))
        }

        #[ink(message)]
        pub fn is_final_decision_maker(&self) -> Result<bool, ()> {
            Ok(self.final_decision_maker == self.env().caller())
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
            let alice = default_accounts().alice;
            set_next_caller(alice);
            let mut betting = Betting::new(alice);

            // not registered yet
            assert_eq!(
                betting.is_registered_as_reviewer(),
                Ok(false),
                "Alice was already registered"
            );

            assert_eq!(
                betting.number_of_reviewers, 0,
                "Smart contract was initialized with at least 1 reviewer."
            );

            // but is final decision maker
            assert_eq!(
                betting.is_final_decision_maker(),
                Ok(true),
                "Alice is not the final decision maker"
            );

            let register = betting.register_as_reviewer();
            assert!(register.is_ok(), "Unable to register Alice.");

            // registered
            assert_eq!(
                betting.is_registered_as_reviewer(),
                Ok(true),
                "Alice is not registered"
            );

            assert_eq!(betting.number_of_reviewers, 1, "Wrong number of reviewers.");
        }
    }
}
