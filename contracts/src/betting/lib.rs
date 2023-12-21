#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod betting {
    use ink::prelude::{string::String, vec::Vec};
    use ink::storage::Mapping;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        // Caller is not a final decision maker
        NotFinalDecisionMaker,
    }

    /// Different states that a bet can be in
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum BetState {
        Created,
        BetAcceptedByBettor2,
        BetRefusedByBettor2,
        EventConcluded,
        Bettor1Wins,
        Bettor2Wins,
        BettorsDisagree,
    }

    /// Information regarding a particular bet
    #[derive(Debug, Default, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub struct Bet {
        /// How much is wagered on the event's outcome
        amount_wagered: Balance,
        /// Who is bettor 1?
        bettor_1: Option<AccountId>,
        /// Who is bettor 2?
        bettor_2: Option<AccountId>,
        /// How is the winner decided?
        criteria_for_winning: Option<String>,
        /// When will the event conclude by (in unix timestamp, milliseconds)
        event_decided_by: u128,
        ///
        state: Option<BetState>,
    }

    #[ink(storage)]
    pub struct Betting {
        /// The amount bettor_1 pays the smart contract to create a bet
        bet_creation_fee: Balance,
        /// A set of registered reviewers
        reviewers: Mapping<AccountId, ()>,
        /// How many registered reviewers there are
        number_of_reviewers: u32,
        /// The accountId that has final say should a reviewer's decision be appealed
        final_decision_maker: AccountId,
        /// Number of bets that have been made
        latest_bet: u32,
        /// A vector of bet information
        bets: Vec<Bet>,
    }

    impl Betting {
        #[ink(constructor)]
        pub fn new(final_decision_maker: AccountId, bet_creation_fee: Balance) -> Self {
            Self {
                bet_creation_fee,
                reviewers: Mapping::default(),
                number_of_reviewers: 0,
                final_decision_maker,
                latest_bet: 0,
                bets: Vec::default(),
            }
        }
        // --------------------------------------------------------
        // Bet-related functions
        // --------------------------------------------------------
        #[ink(message, payable)]
        pub fn create_bet(
            &mut self,
            bettor_2: Option<AccountId>,
            criteria_for_winning: Option<String>,
            event_decided_by: u128,
        ) -> Result<Option<u32>, ()> {
            if self.env().transferred_value() < self.bet_creation_fee {
                return Err(());
            }
            // The amount wagered is calculated based on how much the user sent
            let amount_wagered = self.env().transferred_value() - self.bet_creation_fee;

            let bet = Bet {
                amount_wagered,
                bettor_1: Some(self.env().caller()),
                bettor_2,
                criteria_for_winning,
                event_decided_by,
                state: Some(BetState::Created),
            };

            // update latest bet number
            let bet_number = self.latest_bet;
            self.latest_bet = self.latest_bet + 1;

            self.bets.push(bet);
            Ok(Some(bet_number))
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

        /// Get balance
        #[ink(message)]
        pub fn balance(&self) -> Result<Balance, ()> {
            Ok(self.env().balance())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use chrono::DateTime;

        fn iso8601_to_timestamp_millis(iso8601_string: &str) -> Option<u128> {
            match DateTime::parse_from_rfc3339(iso8601_string) {
                Ok(parsed_time) => {
                    let duration = parsed_time.timestamp_millis();
                    Some(duration as u128)
                }
                Err(_) => None,
            }
        }

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
            let mut betting = Betting::new(alice, 1_u128.pow(10));

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

        #[ink::test]
        fn create_bet() {
            let alice = default_accounts().alice;
            let bob = default_accounts().bob;
            let event_concludes_by = iso8601_to_timestamp_millis("2023-12-21T00:00:00Z").unwrap();

            set_next_caller(alice);

            let fee: Balance = 10;
            let mut betting = Betting::new(alice, fee);
            assert!(betting.balance().unwrap() == 1_000_000); // alice has 1 million to begin with

            let criteria_for_winning: Option<String> =
                Some("Red wins game against blue on December 21st, 2023.".into());

            let amount_sent = 10 * fee;
            let bet_number = ink::env::pay_with_call!(
                betting.create_bet(Some(bob), criteria_for_winning.clone(), event_concludes_by),
                amount_sent
            )
            .unwrap()
            .unwrap();
            assert_eq!(bet_number, 0);

            let bet = betting.bets.get(bet_number as usize).unwrap();
            assert_eq!(bet.bettor_1.unwrap(), alice);
            assert_eq!(bet.bettor_2.unwrap(), bob);
            assert_eq!(bet.criteria_for_winning, criteria_for_winning);
            assert_eq!(bet.event_decided_by, event_concludes_by);
            assert_eq!(bet.state, Some(BetState::Created));
            let amount_wagered = bet.amount_wagered;
            assert_eq!(
                amount_wagered,
                amount_sent - fee,
                "Acutal amount: {amount_wagered}"
            );
        }
    }
}
