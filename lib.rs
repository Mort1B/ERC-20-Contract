#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
    use ink_storage::{traits::SpreadAllocate, Mapping};

    /// Specify the ERC-20 error type
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Return if the balance cannot fulfill a request
        InsufficientBalance,
        /// Return if the allowance cannot fulfill a request
        InsufficientAllowance,
    }

    /// Specify the ERC-20 result type
    pub type Result<T> = core::result::Result<T, Error>;

    /// Creates storage for a simple ERC20 token contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Erc20 {
        /// Total token supply
        total_supply: Balance,
        /// Mapping from owner to number of owned tokens
        balances: Mapping<AccountId, Balance>,
        /// Balances that can be transferred by non-owners: (owner, spender) -> allowed
        allowances: Mapping<(AccountId, AccountId), Balance>,
    }

    /// Emitted when `value` tokens are moved from one account (`from`) to another (`to`).
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Emitted when the allowance of a `spender` for an `owner` is set
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    impl Erc20 {
        /// Creates a new ERC-20 contract with an initial supply.
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            // Initialize mapping for the contract.
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.total_supply = initial_supply;
                let caller = Self::env().caller();
                contract.balances.insert(&caller, &initial_supply);

                // Emit Transfer event
                Self::env().emit_event(Transfer {
                    from: None,
                    to: Some(caller),
                    value: initial_supply,
                })
            })
        }

        /// Returns the total token supply
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        /// private helper function to transfer `value` amount of tokens from account `from` to account `to`.
        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of_impl(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            // Update from balance
            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of_impl(to);
            self.balances.insert(to, &(to_balance + value));

            // Emit Transfer event
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });

            Ok(())
        }

        /// private helper function to get the balance of an account
        #[inline]
        fn balance_of_impl(&self, owner: &AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        /// Function to authorize `spender` to withdraw from your account multiple times, up to the `value` amount.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);

            // Emit Approval event
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_impl(&owner, &spender)
        }

        /// private helper function to get the allowance of an account
        #[inline]
        fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        /// Transfers tokens on the behalf of the `from` account to the `to` account.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&from, &caller);

            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }

            self.transfer_from_to(&from, &to, value)?;
            self.allowances
                .insert((&from, &caller), &(allowance - value));
            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        use ink_env::AccountId;
        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// Test if the default constructor does its job.
        #[ink::test]
        fn new_works() {
            let contract = Erc20::new(777);
            assert_eq!(contract.total_supply(), 777);
        }

        /// We if balance works
        #[ink::test]
        fn balance_works() {
            let contract = Erc20::new(100);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
        }

        /// Test if transfer works
        #[ink::test]
        fn transfer_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
            assert_eq!(contract.transfer(AccountId::from([0x0; 32]), 50), Ok(()));
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 50);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 50);
        }
        #[ink::test]
        fn transfer_from_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract
                .approve(AccountId::from([0x1; 32]), 20)
                .unwrap_or_default();
            contract
                .transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 10)
                .unwrap_or_default();
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
        }

        #[ink::test]
        fn allowance_works() {
            let mut contract = Erc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            contract
                .approve(AccountId::from([0x1; 32]), 200)
                .unwrap_or_default();
            assert_eq!(
                contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                200
            );
            contract
                .transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 50)
                .unwrap_or_default();
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 50);
            assert_eq!(
                contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                150
            );

            contract
                .transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 100)
                .unwrap_or_default();
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 50);
            assert_eq!(
                contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                150
            );
        }
    }
}
