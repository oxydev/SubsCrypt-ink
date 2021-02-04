#![cfg_attr(not(feature = "std"), no_std)]

//#![feature(type_ascription)]
use hex_literal::hex;
use sha2::{Sha256, Sha512, Digest};
use ink_lang as ink;

use ink_storage::collections::HashMap;

#[ink::contract]
mod subscrypt {
    use ink_storage::{collections};
    use ink_storage::collections::HashMap;
    use ink_primitives::Key;
    use ink_env::{Error, AccountId as Account};
    use sha2::Sha256;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        traits::{
            PackedLayout,
            SpreadLayout,
        },
        Lazy,
    };
    use std::convert::TryInto;

    #[derive(SpreadLayout, PackedLayout, scale::Encode, scale::Decode, Debug, scale_info::TypeInfo)]
    struct SubscriptionRecord {
        provider: Account,
        plan: PlanConsts,
        plan_index: u128,
        subscription_time: u64,
        meta_data_encrypted: String,
        //encrypted Data with public key of provider
        refunded: bool,
    }

    #[derive(SpreadLayout, PackedLayout, scale::Encode, scale::Decode, Debug, scale_info::TypeInfo)]
    struct PlanRecord {
        subscription_records: Vec<SubscriptionRecord>,
        pass_hash: String,
    }

    #[derive(PackedLayout, SpreadLayout, scale::Encode, scale::Decode, Debug, scale_info::TypeInfo)]
    struct PlanConsts {
        duration: u64,
        active_session_limit: u128,
        price: u128,
        max_refund_percent_policy: u128,
        disabled: bool,
    }

    #[derive(PackedLayout, SpreadLayout, scale::Encode, scale::Decode, Debug, scale_info::TypeInfo)]
    struct Provider {
        plans: Vec<PlanConsts>,
        money_address: Account,
        payment_manager: LinkedList,
    }

    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout, Debug, scale_info::TypeInfo)]
    #[cfg_attr(
    feature = "std",
    )]
    struct User {
        list_of_providers: Vec<Account>,
        joined_time: u64,
        subs_crypt_pass_hash: String,
    }

    #[derive(PackedLayout, SpreadLayout, scale::Encode, scale::Decode, Debug, scale_info::TypeInfo)]
    struct LinkedList {
        head: u128,
        back: u128,
        length: u128,
    }

    #[derive(PackedLayout, SpreadLayout, scale::Encode, scale::Decode, Debug, scale_info::TypeInfo)]
    struct Object {
        number: u128,
        next_day: u128,
    }

    #[ink(storage)]
    // #[derive(PackedLayout,scale::Encode,scale::Decode,scale_info::TypeInfo)]
    pub struct Subscrypt {
        start_time: u64,
        provider_register_fee: u128,
        providers: HashMap<Account, Provider>,
        objects: HashMap<(Account, u128), Object>,
        users: HashMap<Account, User>,
        records: HashMap<(Account, Account), PlanRecord>,
        // first account is user the next one is provider
        plan_index_to_record_index: HashMap<(Account, Account, u128), u128>,
    }

    impl Subscrypt {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                start_time: Self::env().block_timestamp(),
                provider_register_fee: 100,
                providers: ink_storage::collections::HashMap::new(),
                users: ink_storage::collections::HashMap::new(),
                objects: ink_storage::collections::HashMap::new(),
                records: ink_storage::collections::HashMap::new(),
                plan_index_to_record_index: ink_storage::collections::HashMap::new(),
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                start_time: 0,
                provider_register_fee: 0,
                providers: Default::default(),
                users: Default::default(),
                objects: Default::default(),
                records: Default::default(),
                plan_index_to_record_index: Default::default(),
            }
        }

        #[ink(message)]
        pub fn provider_register(&mut self, durations: Vec<u64>, active_session_limits: Vec<u128>, prices: Vec<u128>, max_refund_percent_policies: Vec<u128>, address: Account) {
            let caller = self.env().caller();
            assert!(self.env().transferred_balance() >= self.provider_register_fee, "You have to pay a minimum amount to register in the contract!");
            assert!(!self.providers.contains_key(&caller), "You can not register again in the contract!");

            let mut provider = Provider {
                plans: Vec::new(),
                money_address: address,
                payment_manager: LinkedList::new(),
            };

            self.providers.insert(caller, provider);
            for i in 0..durations.len() {
                let cons = PlanConsts {
                    duration: durations[i],
                    active_session_limit: active_session_limits[i],
                    price: prices[i],
                    max_refund_percent_policy: max_refund_percent_policies[i],
                    disabled: false,
                };
                provider.plans.push(cons);
            }
        }

        #[ink(message)]
        pub fn add_plan(&mut self, durations: Vec<u64>, active_session_limits: Vec<u128>, prices: Vec<u128>, max_refund_percent_policies: Vec<u128>) {
            let caller = self.env().caller();
            assert!(self.providers.contains_key(&caller), "You should first register in the contract!");
            let mut provider = self.providers.get_mut(&caller).unwrap();
            for i in 0..durations.len() {
                let cons = PlanConsts {
                    duration: durations[i],
                    active_session_limit: active_session_limits[i],
                    price: prices[i],
                    max_refund_percent_policy: max_refund_percent_policies[i],
                    disabled: false,
                };
                provider.plans.push(cons);
            }
        }


        #[ink(message)]
        #[feature(type_ascription)]
        pub fn edit_plan(&mut self, plan_index: u128, duration: u64, active_session_limit: u128, price: u128, max_refund_percent_policy: u128, disabled: bool) {
            let number: usize = plan_index.try_into().unwrap();
            let caller = self.env().caller();
            assert!(self.providers.get(&caller).unwrap().plans.len() > plan_index.try_into().unwrap(), "please select a valid plan");
            let mut provider = self.providers.get_mut(&caller).unwrap();
            // let mut plan: PlanConsts = provider.plans.get_mut(plan_index.try_into().unwrap());
            let mut plan: PlanConsts = *(provider.plans.get(number).unwrap());

            plan.duration = duration;
            plan.active_session_limit = active_session_limit;
            plan.price = price;
            plan.max_refund_percent_policy = max_refund_percent_policy;
            plan.disabled = disabled;
        }

        #[ink(message)]
        pub fn change_disable(&mut self, plan_index: u128) {
            let caller = self.env().caller();
            let number: usize = plan_index.try_into().unwrap();
            assert!(self.providers.get(&caller).unwrap().plans.len() > plan_index.try_into().unwrap(), "please select a valid plan");
            let x = self.providers.get(&caller).unwrap().plans[number].disabled;
            self.providers.get(&caller).unwrap().plans[number].disabled = !x;
        }

        #[ink(message)]
        pub fn subscribe(&mut self, provider_address: Account, plan_index: u128, pass: String, metadata: String) {
            let caller: Account = self.env().caller();
            if !self.users.contains_key(&caller) {
                self.users.insert(caller, User {
                    list_of_providers: Vec::new(),
                    joined_time: self.env().block_timestamp(),
                    subs_crypt_pass_hash: "".to_string(),
                });
            }
            let mut user: &mut User = self.users.get_mut(&caller).unwrap();
            let number: usize = plan_index.try_into().unwrap();

            assert!(self.providers.contains_key(&provider_address), "Provider not existed in the contract!");
            assert!(self.providers.get(&provider_address).unwrap().plans.len() > plan_index.try_into().unwrap(), "Wrong plan index!");
            let consts: PlanConsts = self.providers.get(&provider_address).unwrap().plans[number];

            assert_eq!(consts.price, self.env().transferred_balance(), "You have to pay exact plan price");
            assert!(!consts.disabled, "Plan is currently disabled by provider");
            //assert!(!self.check_subscription(self, caller, provider_address, plan_index.copy()), "You are already subscribed to this plan!");

            if !self.records.contains_key(&(caller, provider_address)) {
                user.list_of_providers.push(provider_address);
            }

            let mut plan_record: &mut PlanRecord = self.records.get_mut(&(caller, provider_address)).unwrap();
            self.plan_index_to_record_index.insert((caller, provider_address, plan_index), self.records.get(&(caller, provider_address)).unwrap().subscription_records.len().try_into().unwrap());

            let record: SubscriptionRecord = SubscriptionRecord {
                provider: provider_address,
                plan: PlanConsts {
                    duration: consts.duration,
                    active_session_limit: consts.active_session_limit,
                    price: consts.price,
                    max_refund_percent_policy: consts.max_refund_percent_policy,
                    disabled: consts.disabled,
                },
                plan_index,
                subscription_time: self.env().block_timestamp(),
                meta_data_encrypted: metadata,
                refunded: false,
            };
            plan_record.subscription_records.push(record);

            plan_record.pass_hash = pass;

            let addr: Account = self.providers.get(&provider_address).unwrap().money_address;
            // send money to money_address (1000 - plan.max_refund_percent_policy) / 1000;
            self.transfer(self.env().caller(), consts.price * (1000 - consts.max_refund_percent_policy) / 1000);

            self.providers.get_mut(&provider_address).unwrap().payment_manager.add_entry((self.env().block_timestamp() + consts.duration - &self.start_time) / 86400, (self.env().transferred_balance() * consts.max_refund_percent_policy) / 1000);
        }

        #[ink(message)]
        pub fn set_subscrypt_pass(&mut self, pass: String) {
            assert!(self.users.contains_key(&self.env().caller()));
            self.users.get_mut(&self.env().caller()).unwrap().subs_crypt_pass_hash = pass;
        }

        #[ink(message)]
        pub fn withdraw(&mut self) -> u128 {
            assert!(self.providers.contains_key(&self.env().caller()), "You are not a registered provider");
            let caller: Account = self.env().caller();

            let paid: u128 = self.providers.get_mut(&caller).unwrap().payment_manager.process((self.env().block_timestamp() / 86400).try_into().unwrap());
            if paid > 0 {
                self.transfer(caller, paid);

            }
            return paid;
        }

        #[ink(message)]
        pub fn refund(&mut self, provider_address: Account, plan_index: u128) {
            let caller: Account = self.env().caller();
            // assert!(self.check_subscription(caller, provider_address, plan_index));
            let last_index: &u128 = self.plan_index_to_record_index.get(&(caller,provider_address,plan_index)).unwrap();
            let mut record: SubscriptionRecord = self.users.get(&caller).unwrap().records.get(&provider_address).unwrap().subscriptionRecords.get(last_index).unwrap();
            let mut time_percent: u128 = (self.env().block_timestamp() - record.subscription_time) * 1000 / (record.plan.duration);
            if 1000 - time_percent > record.plan.max_refund_percent_policy {
                time_percent = record.plan.max_refund_percent_policy;
            } else {
                time_percent = 1000 - time_percent.copy();
            }
            let transfer_value: u128 = time_percent * record.plan.price / 1000;
            record.refunded = true;
            self.transfer(caller, transfer_value);
            if time_percent < record.plan.max_refund_percent_policy {
                let refunded_amount: u128 = (record.plan.max_refund_percent_policy.copy() - time_percent.copy()) * record.plan.price.copy() / 1000;
                self.transfer(self.providers.get(&provider_address).unwrap().money_address, transfer_value.copy());
            }

            self.providers.get_mut(&provider_address).unwrap().payment_manager.remove_entry((record.plan.duration.copy() + record.subscription_time.copy() - &self.start_time) / 86400, record.plan.price.copy() * record.plan.max_refund_percent_policy.copy());
        }

        #[ink(message)]
        pub fn check_auth(&self, user: Account, provider: Account, token: String, pass_phrase: String) {
            // let mut hasher = Sha256::new();
            // hasher.update(b"hello world");
            // let result = hasher.finalize();
        }

        #[ink(message)]
        pub fn retrieve_whole_data_with_password(&self) {
            self.my_value_or_zero(&self.env().caller())
        }

        #[ink(message)]
        pub fn retrieve_whole_data_with_wallet(&self) {
            self.my_value_or_zero(&self.env().caller())
        }

        #[ink(message)]
        pub fn retrieve_data_with_password(&self) {
            self.my_value_or_zero(&self.env().caller())
        }

        #[ink(message)]
        pub fn retrieve_data_with_wallet(&self) {
            self.my_value_or_zero(&self.env().caller())
        }

        fn check_subscription(str: &Subscrypt, caller: Account, provider_address: Account, plan_index: u128) {
            unimplemented!()
        }

        fn transfer(&self, addr: Account, amount: u128) {
            self.env()
                .transfer(addr, amount)
                .map_err(|err| {
                    match err {
                        ink_env::Error::BelowSubsistenceThreshold => {
                            Error::BelowSubsistenceThreshold
                        }
                        _ => Error::TransferFailed,
                    }
                });
        }
    }


    impl LinkedList {
        pub fn new() -> Self {
            Self {
                back: 0,
                head: 0,
                length: 0,
            }
        }

        pub fn add_entry(&mut self, day_id: u64, amount: u128) {
            if self.length == 0 {
                let object = Object { number: amount, next_day: day_id.copy() };
                self.head = day_id.copy();
                self.objects.insert(day_id.copy(), object);
                self.back = day_id;
                self.length = self.length.copy() + 1;
            } else if day_id < self.head {
                let object = Object { number: amount, next_day: day_id.copy() };
                self.head = day_id.copy();
                self.objects.insert(day_id.copy(), object);
                self.length = self.length.copy() + 1;
            } else if day_id > self.back {
                self.objects.get(&day_id).unwrap().nextDay = day_id.copy();
                let object = Object { number: amount, next_day: day_id.copy() };
                self.back = day_id.copy();
                self.objects.insert(day_id.copy(), object);
                self.length = self.length.copy() + 1;
            } else {
                let mut cur_id: u128 = self.head.copy();
                loop {
                    if day_id == cur_id {
                        self.objects.get(&day_id).number += amount;
                        break;
                    } else if day_id < self.objects.get(&cur_id).unwrap().next_day {
                        let object = Object { number: amount, next_day: self.objects.get(&cur_id).unwrap().next_day };
                        self.objects.get(&cur_id).next_day = day_id.copy();
                        self.objects.insert(day_id.copy(), object);
                        self.length = self.length.copy() + 1;
                        break;
                    }
                    cur_id = self.objects.get(&cur_id).next_day;
                    if cur_id == self.back {
                        break;
                    }
                }
            }
        }

        pub fn remove_entry(&mut self, day_id: u128, amount: u128) {
            self.objects.get(&day_id).number -= amount;
        }

        pub fn process(&mut self, day_id: u64) -> u128 {
            let mut sum: u128 = 0;
            let mut cur_id: u128 = self.head.copy();
            while day_id >= cur_id {
                sum += self.objects.get(&cur_id).unwrap().number;
                cur_id = self.objects.get(&cur_id).unwrap().next_day;
                self.length -= 1;
                if cur_id == self.back {
                    break;
                }
            }
            self.head = cur_id;
            return sum;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[test]
        fn constructor_works() {
            let subsCrypt = Subscrypt::new();
            assert_eq!(subsCrypt.provider_register_fee, 100);
        }

        #[test]
        fn default_works() {
            let subsCrypt = Subscrypt::default();
            assert_eq!(subsCrypt.provider_register_fee, 0);
        }
        // pub fn provider_register(&mut self, durations: Vec<u128>, active_session_limits: Vec<u128>, prices: Vec<u128>, max_refund_percent_policies: Vec<u128>, address: AccountId) {

        #[test]
        fn provider_register_works() {
            let mut subsCrypt = Subscrypt::new();
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            subsCrypt.provider_register(Vec::new(60 * 60 * 24, 60 * 60 * 24 * 30), Vec::new(2, 2), Vec::new(10000, 50000), Vec::new(50, 100), accounts.alice);
            assert_eq!(subsCrypt.providers.get(accounts.alice).unwrap().plans.get(0).duration, 60 * 60 * 24);
            // contract.inc(-50);
            // assert_eq!(-3, -3);
        }

        #[ink::test]
        fn my_value_works() {
            // let mut contract = Incrementer::new(11);
            // assert_eq!(contract.get(), 11);
            // assert_eq!(contract.get_mine(), 0);
            // contract.inc_mine(5);
            // assert_eq!(contract.get_mine(), 5);
            // contract.inc_mine(10);
            assert_eq!(1, 15);
        }
    }
}
