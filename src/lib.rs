// Copyright 2020-2021 OxyDev.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

#[ink_lang::contract]
pub mod subscrypt {
    use ink_storage::collections::HashMap;
    use ink_env::{Error as Er, AccountId as Account, Error};
    use ink_env::hash::{Sha2x256};
    use ink_prelude::vec::Vec;
    use ink_storage::{
        traits::{
            PackedLayout,
            SpreadLayout,
        },
    };
    use core::convert::TryInto;
    use ink_prelude::string::String;

    /// This struct represents a subscription record
    /// # fields:
    /// * provider
    /// * plan
    /// * plan_index
    /// * subscription_time : this stores start time of each subscription (used in linkedList)
    /// * meta_data_encrypted
    /// * refunded
    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout, Debug, scale_info::TypeInfo)]
    pub struct SubscriptionRecord {
        pub provider: Account,
        pub plan: PlanConsts,
        pub plan_index: u128,
        subscription_time: u64,
        meta_data_encrypted: String,
        //encrypted Data with public key of provider
        pub refunded: bool,
    }

    /// This struct stores user plan records
    /// # fields:
    /// * subscription_records
    /// * pass_hash : hash of (token + pass_phrase) for authenticating user without wallet
    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout, Debug, scale_info::TypeInfo)]
    pub struct PlanRecord {
        pub subscription_records: Vec<SubscriptionRecord>,
        pass_hash: [u8; 32],
    }

    /// This struct stores configs of plan which is set by provider
    #[derive(scale::Encode, scale::Decode, PackedLayout, SpreadLayout, Debug, scale_info::TypeInfo, Clone, Copy)]
    pub struct PlanConsts {
        pub duration: u64,
        pub(crate) active_session_limit: u128,
        pub(crate) price: u128,
        pub(crate) max_refund_percent_policy: u128,
        pub disabled: bool,
    }

    /// This struct represents a provider
    /// # fields:
    /// * plans
    /// * money_address : provider earned money will be sent to this address
    /// * payment_manager : struct for handling refund requests
    #[derive(scale::Encode, scale::Decode, PackedLayout, SpreadLayout, Debug, scale_info::TypeInfo)]
    pub struct Provider {
        pub(crate) plans: Vec<PlanConsts>,
        pub(crate) money_address: Account,
        payment_manager: LinkedList,
    }

    /// This struct represents a user
    /// # fields:
    /// * list_of_providers : list of providers that the user subscribed to
    /// * subs_crypt_pass_hash : pass hash for retrieve data in subscrypt user dashboard
    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout, Debug, scale_info::TypeInfo)]
    pub struct User {
        pub list_of_providers: Vec<Account>,
        pub subs_crypt_pass_hash: [u8; 32],
    }

    /// Struct for handling payments of refund
    /// # Description
    ///
    /// This LinkedList is used for keeping tracking of each subscription that will end in some
    /// specific date in future. We order these subscriptions by their date of expiration, so we
    /// will be able to easily calculate and handle refund - withdraw methods with a minimum
    /// transaction fee. Each entity of the linked-list is `PaymentAdmission` struct.
    #[derive(scale::Encode, scale::Decode, PackedLayout, SpreadLayout, Debug, scale_info::TypeInfo)]
    pub struct LinkedList {
        pub head: u64,
        pub back: u64,
        pub length: u128,
    }

    /// Struct that represents amount of money that can be withdraw after its due date passed.
    #[derive(scale::Encode, scale::Decode, PackedLayout, SpreadLayout, Debug, scale_info::TypeInfo)]
    struct DailyLockedAmount {
        amount: u128,
        next_day: u64,
    }

    /// Main struct of contract
    /// # fields:
    /// * `start_time` : start time of the contract which is used in `LinkedList`
    /// * `provider_register_fee`
    /// * `providers` : the hashmap that stores providers data
    /// * `users` : the hashmap that stores users data
    /// * `daily_locked_amounts` : the hashmap that stores `DailyLockedAmount` data of each day in order
    /// * `records` : the hashmap that stores user's subscription records data
    /// * `plan_index_to_record_index` : the hashmap that stores user's last `SubscriptionRecord` index
    /// in `PlanRecord.subscription_records` for each (user, provider, plan_index)
    #[ink(storage)]
    pub struct Subscrypt {
        start_time: u64,
        pub provider_register_fee: u128,
        // (provider AccountId) -> provider data
        pub(crate) providers: HashMap<Account, Provider>,
        // (user AccountId) -> user data
        pub users: HashMap<Account, User>,
        // (provider AccountId , day_id) -> payment admission
        daily_locked_amounts: HashMap<(Account, u64), DailyLockedAmount>,
        // (user AccountId, provider AccountId) -> PlanRecord struct
        pub records: HashMap<(Account, Account), PlanRecord>,
        // (user AccountId, provider AccountId, plan_index) -> index
        plan_index_to_record_index: HashMap<(Account, Account, u128), u128>,
    }

    impl Default for Subscrypt {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Subscrypt {
        #[ink(constructor)]
        pub fn new() -> Self {
            Subscrypt::default()
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                start_time: Self::env().block_timestamp(),
                provider_register_fee: 100,
                providers: HashMap::new(),
                users: ink_storage::collections::HashMap::new(),
                daily_locked_amounts: ink_storage::collections::HashMap::new(),
                records: ink_storage::collections::HashMap::new(),
                plan_index_to_record_index: ink_storage::collections::HashMap::new(),
            }
        }

        /// Registering a new `Provider` by paying the required fee amount (`provider_register_fee`)
        ///
        /// # Panics
        ///
        /// If paid amount is less than `provider_register_fee`
        /// if same `AccountId` registered as provider previously.
        ///
        /// # Examples
        /// Examples of different situations in `provider_register_works` , `provider_register_works2` and
        /// `provider_register_works3` in `tests/test.rs`
        #[ink(message, payable)]
        pub fn provider_register(&mut self, durations: Vec<u64>, active_session_limits: Vec<u128>, prices: Vec<u128>, max_refund_percent_policies: Vec<u128>, address: Account) {
            let caller = self.env().caller();
            assert!(self.env().transferred_balance() >= self.provider_register_fee, "You have to pay a minimum amount to register in the contract!");
            assert!(!self.providers.contains_key(&caller), "You can not register again in the contract!");

            let provider = Provider {
                plans: Vec::new(),
                money_address: address,
                payment_manager: LinkedList::new(),
            };
            self.providers.insert(caller, provider);

            self.add_plan(durations, active_session_limits, prices, max_refund_percent_policies);
        }


        /// add_plan : add plans to provider storage
        /// # arguments:
        /// * durations
        /// * active_session_limits
        /// * prices
        /// * max_refund_percent_policies
        #[ink(message)]
        pub fn add_plan(&mut self, durations: Vec<u64>, active_session_limits: Vec<u128>, prices: Vec<u128>, max_refund_percent_policies: Vec<u128>) {
            assert_eq!(durations.len(), active_session_limits.len());
            assert_eq!(prices.len(), active_session_limits.len());
            assert_eq!(max_refund_percent_policies.len(), active_session_limits.len());

            let caller = self.env().caller();
            assert!(self.providers.contains_key(&caller), "You should first register in the contract!");
            let provider = self.providers.get_mut(&caller).unwrap();
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

        /// Editing previously created plans of the `caller`
        ///
        /// # Panics
        ///
        /// If `plan_index` is bigger than the length of `plans` of `provider`
        ///
        /// # Examples
        /// Examples of different situations in `edit_plan_works` and `edit_plan_works2` in `tests/test.rs`
        #[ink(message)]
        pub fn edit_plan(&mut self, plan_index: u128, duration: u64, active_session_limit: u128, price: u128, max_refund_percent_policy: u128, disabled: bool) {
            let number: usize = plan_index.try_into().unwrap();
            let caller = self.env().caller();
            assert!(self.providers.get(&caller).unwrap().plans.len() > plan_index.try_into().unwrap(), "please select a valid plan");
            let provider = self.providers.get_mut(&caller).unwrap();
            let mut plan: &mut PlanConsts = provider.plans.get_mut(number).unwrap();

            plan.duration = duration;
            plan.active_session_limit = active_session_limit;
            plan.price = price;
            plan.max_refund_percent_policy = max_refund_percent_policy;
            plan.disabled = disabled;
        }

        /// change_disable : disable and enable a plan
        /// # arguments:
        /// * plan_index
        #[ink(message)]
        pub fn change_disable(&mut self, plan_index: u128) {
            let caller = self.env().caller();
            let number: usize = plan_index.try_into().unwrap();
            assert!(self.providers.get(&caller).unwrap().plans.len() > plan_index.try_into().unwrap(), "please select a valid plan");
            let x = self.providers.get(&caller).unwrap().plans[number].disabled;
            self.providers.get_mut(&caller).unwrap().plans[number].disabled = !x;
        }

        /// subscribe : users call this function to subscribe a plan
        /// # arguments:
        /// * provider_address
        /// * plan_index
        /// * pass : hash of (token + pass_phrase)
        /// * metadata : extra metadata of the plan
        #[ink(message, payable)]
        pub fn subscribe(&mut self, provider_address: Account, plan_index: u128, pass: [u8; 32], metadata: String) {
            let caller: Account = self.env().caller();
            let time: u64 = self.env().block_timestamp();
            let value: u128 = self.env().transferred_balance();
            let number: usize = plan_index.try_into().unwrap();
            let consts: PlanConsts = self.providers.get(&provider_address).unwrap().plans[number];

            assert_eq!(consts.price, value, "You have to pay exact plan price");
            assert!(!consts.disabled, "Plan is currently disabled by provider");

            assert!(!self.check_subscription(caller, provider_address, plan_index), "You are already subscribed to this plan!");
            assert!(self.providers.contains_key(&provider_address), "Provider not existed in the contract!");
            assert!(self.providers.get(&provider_address).unwrap().plans.len() > plan_index.try_into().unwrap(), "Wrong plan index!");
            if !self.users.contains_key(&caller) {
                self.users.insert(caller, User {
                    list_of_providers: Vec::new(),
                    subs_crypt_pass_hash: pass,
                });
            }

            let user: &mut User = self.users.get_mut(&caller).unwrap();
            if !self.records.contains_key(&(caller, provider_address)) {
                user.list_of_providers.push(provider_address);
                self.records.insert((caller, provider_address), PlanRecord {
                    subscription_records: Vec::new(),
                    pass_hash: pass,
                });
            }
            let plan_record: &mut PlanRecord = self.records.get_mut(&(caller, provider_address)).unwrap();
            self.plan_index_to_record_index.insert((caller, provider_address, plan_index), plan_record.subscription_records.len().try_into().unwrap());
            let record: SubscriptionRecord = SubscriptionRecord {
                provider: provider_address,
                plan: consts,
                plan_index,
                subscription_time: time,
                meta_data_encrypted: metadata,
                refunded: false,
            };
            plan_record.subscription_records.push(record);

            let addr: &Account = &self.providers.get(&provider_address).unwrap().money_address;
            // send money to money_address (1000 - plan.max_refund_percent_policy) / 1000;
            assert_eq!(self.transfer(*addr, consts.price * (1000 - consts.max_refund_percent_policy) / 1000), Ok(()));
            let start_time = self.start_time;
            let block_time = self.env().block_timestamp();
            let transferred_balance = self.env().transferred_balance();
            self.add_entry(provider_address, (block_time + consts.duration - start_time) / 86400, (transferred_balance * consts.max_refund_percent_policy) / 1000)
        }

        /// set_subscrypt_pass : users can change their pass_hash
        /// # arguments:
        /// * pass : hash of (token + pass_phrase)
        #[ink(message)]
        pub fn set_subscrypt_pass(&mut self, pass: [u8; 32]) {
            assert!(self.users.contains_key(&self.env().caller()));
            self.users.get_mut(&self.env().caller()).unwrap().subs_crypt_pass_hash = pass;
        }

        /// withdraw : providers call this function to claim their unlocked money
        #[ink(message)]
        pub fn withdraw(&mut self) -> u128 {
            assert!(self.providers.contains_key(&self.env().caller()), "You are not a registered provider");
            let caller: Account = self.env().caller();
            let paid: u128 = self.process(caller, self.env().block_timestamp() / 86400);
            if paid > 0 {
                assert_eq!(self.transfer(caller, paid), Ok(()));
            }
            paid
        }

        /// refund : users can refund their money back
        /// # arguments:
        /// * provider_address
        /// * plan_index
        #[ink(message)]
        pub fn refund(&mut self, provider_address: Account, plan_index: u128) {
            let caller: Account = self.env().caller();
            let time: u64 = self.env().block_timestamp();
            assert!(self.check_subscription(caller, provider_address, plan_index), "You are not in this plan or already refunded");
            assert!(self.plan_index_to_record_index.contains_key(&(caller, provider_address, plan_index)));
            let last_index: &u128 = self.plan_index_to_record_index.get(&(caller, provider_address, plan_index)).unwrap();
            let number: usize = (*last_index).try_into().unwrap();
            let record: &SubscriptionRecord = self.records.get(&(caller, provider_address)).unwrap().subscription_records.get(number).unwrap();

            // it shows how much of your subscription plan is passed in range of 0 to 1000 and more (if you refund after the plan is finished)
            let spent_time_percent: u128 = ((time - record.subscription_time) * 1000 / (record.plan.duration)).try_into().unwrap();
            // to avoid refund after plan is finished
            assert!(spent_time_percent <= 1000);

            // amount of time that is remained till the end of your plan = 1000 - spent_time_percent

            let mut remained_time_percent = 1000 - spent_time_percent;
            if remained_time_percent > record.plan.max_refund_percent_policy {
                // in this case the customer wants to refund very early so he want to get
                // more than the amount of refund policy, so we can only give back just
                // max_refund_percent_policy of his/her subscription. Whole locked money will go directly to
                // account of the customer
                remained_time_percent = record.plan.max_refund_percent_policy;
            } else {
                // in this case the customer wants to refund, but he/she used most of his subscription time
                // and now he/she will get portion of locked money, and the provider will get the rest of money
                let provider_portion_locked_money: u128 = (record.plan.max_refund_percent_policy - remained_time_percent) * record.plan.price / 1000;
                assert_eq!(self.transfer(self.providers.get(&provider_address).unwrap().money_address, provider_portion_locked_money), Ok(()));
            }

            let customer_portion_locked_money: u128 = remained_time_percent * record.plan.price / 1000;
            assert_eq!(self.transfer(caller, customer_portion_locked_money), Ok(()));


            let passed_time = record.plan.duration + record.subscription_time - self.start_time;
            let amount = record.plan.price * record.plan.max_refund_percent_policy;
            self.remove_entry(provider_address, passed_time / 86400, amount / 1000);
            self.records.get_mut(&(caller, provider_address)).unwrap().subscription_records.get_mut(number).unwrap().refunded = true;
        }

        /// check_auth : this function authenticates the user with token and pass_phrase
        /// # arguments:
        /// * user
        /// * provider
        /// * plan_index
        /// * token and pass_phrase : these are for authenticating
        #[ink(message)]
        pub fn check_auth(&self, user: Account, provider: Account, token: String, pass_phrase: String) -> bool {
            if !self.records.contains_key(&(user, provider)) {
                return false;
            }
            let encodable = [
                token,
                pass_phrase
            ];
            let encoded = self.env().hash_encoded::<Sha2x256, _>(&encodable);
            if encoded == self.records.get(&(user, provider)).unwrap().pass_hash {
                return true;
            }
            false
        }

        /// retrieve_whole_data_with_password : retrieve all user data when wallet is not available.
        /// # arguments:
        /// * user
        /// * token and phrase : subscrypt passphrase
        /// # return value : vector of subscription records
        #[ink(message)]
        pub fn retrieve_whole_data_with_password(&self, user: Account, token: String, phrase: String) -> Vec<SubscriptionRecord> {
            let encodable = [
                token,
                phrase
            ];
            let encoded = self.env().hash_encoded::<Sha2x256, _>(&encodable);
            assert_eq!(encoded, self.users.get(&user).unwrap().subs_crypt_pass_hash, "Wrong auth");
            self.retrieve_whole_data(user)
        }

        /// retrieve_whole_data_with_wallet : retrieve all user data.
        /// # return value : vector of subscription records
        #[ink(message)]
        pub fn retrieve_whole_data_with_wallet(&self) -> Vec<SubscriptionRecord> {
            let caller: Account = self.env().caller();
            self.retrieve_whole_data(caller)
        }

        fn retrieve_whole_data(&self, caller: Account) -> Vec<SubscriptionRecord> {
            assert!(self.users.contains_key(&caller));
            let mut data: Vec<SubscriptionRecord> = Vec::new();
            let user: &User = self.users.get(&caller).unwrap();
            for i in 0..user.list_of_providers.len() {
                let plan_records: &PlanRecord = self.records.get(&(caller, user.list_of_providers[i])).unwrap();
                for i in 0..plan_records.subscription_records.len() {
                    let k = SubscriptionRecord {
                        provider: plan_records.subscription_records[i].provider,
                        plan: plan_records.subscription_records[i].plan,
                        plan_index: plan_records.subscription_records[i].plan_index,
                        subscription_time: plan_records.subscription_records[i].subscription_time,
                        meta_data_encrypted: plan_records.subscription_records[i].meta_data_encrypted.clone(),
                        refunded: plan_records.subscription_records[i].refunded,
                    };
                    data.push(k);
                }
            }
            data
        }

        /// retrieve_data_with_password : retrieve user data when wallet is not available.
        /// # arguments:
        /// * user
        /// * provider_address
        /// * token and phrase
        /// # return value: vector of subscription records from a specific provider
        #[ink(message)]
        pub fn retrieve_data_with_password(&self, user: Account, provider_address: Account, token: String, phrase: String) -> Vec<SubscriptionRecord> {
            let encodable = [
                token,
                phrase
            ];
            let encoded = self.env().hash_encoded::<Sha2x256, _>(&encodable);
            assert_eq!(encoded, self.records.get(&(user, provider_address)).unwrap().pass_hash, "Wrong auth");
            self.retrieve_data(user, provider_address)
        }

        /// retrieve_data_with_wallet : retrieve user data whit wallet.
        /// # arguments:
        /// * provider_address
        /// # return value: vector of subscription records from a specific provider
        #[ink(message)]
        pub fn retrieve_data_with_wallet(&self, provider_address: Account) -> Vec<SubscriptionRecord> {
            let caller: Account = self.env().caller();
            self.retrieve_data(caller, provider_address)
        }

        fn retrieve_data(&self, caller: Account, provider_address: Account) -> Vec<SubscriptionRecord> {
            assert!(self.users.contains_key(&caller));
            assert!(self.records.contains_key(&(caller, provider_address)));
            let mut data: Vec<SubscriptionRecord> = Vec::new();

            let plan_records: &PlanRecord = self.records.get(&(caller, provider_address)).unwrap();
            for i in 0..plan_records.subscription_records.len() {
                let k = SubscriptionRecord {
                    provider: plan_records.subscription_records[i].provider,
                    plan: plan_records.subscription_records[i].plan,
                    plan_index: plan_records.subscription_records[i].plan_index,
                    subscription_time: plan_records.subscription_records[i].subscription_time,
                    meta_data_encrypted: plan_records.subscription_records[i].meta_data_encrypted.clone(),
                    refunded: plan_records.subscription_records[i].refunded,
                };
                data.push(k);
            }
            data
        }


        /// check_subscription : provides can use this function to check if user has authority to plan or not
        /// # arguments:
        /// * user : user address
        /// * provider_address
        /// * plan_index
        /// # return value: boolean that indicates the user has authority or not
        #[ink(message)]
        pub fn check_subscription(&self, user: Account, provider_address: Account, plan_index: u128) -> bool {
            if !self.users.contains_key(&user) {
                return false;
            }
            if !self.records.contains_key(&(user, provider_address)) {
                return false;
            }
            if !self.plan_index_to_record_index.contains_key(&(user, provider_address, plan_index)) {
                return false;
            }
            let last_index: u128 = *self.plan_index_to_record_index.get(&(user, provider_address, plan_index)).unwrap();
            let number: usize = last_index.try_into().unwrap();
            let record: &SubscriptionRecord = &self.records.get(&(user, provider_address)).unwrap().subscription_records[number];
            if record.plan_index != plan_index || record.refunded || record.plan.duration + record.subscription_time < self.env().block_timestamp() {
                return false;
            }
            true
        }

        fn transfer(&self, addr: Account, amount: u128) -> Result<(), Error> {
            self.env()
                .transfer(addr, amount)
                .map_err(|err| {
                    match err {
                        ink_env::Error::BelowSubsistenceThreshold => {
                            Er::BelowSubsistenceThreshold
                        }
                        _ => Er::TransferFailed,
                    }
                })
        }

        /// add_entry : add a payment entry to provider payment management linked list
        /// # arguments:
        /// * provider_address
        /// * day_id : the calculation formula is : (finish date - contract start date) / 86400
        /// * amount : money amount
        fn add_entry(&mut self, provider_address: Account, day_id: u64, amount: u128) {
            let linked_list: &mut LinkedList = &mut self.providers.get_mut(&provider_address).unwrap().payment_manager;
            if linked_list.length == 0 {
                let object = DailyLockedAmount { amount, next_day: day_id };
                linked_list.head = day_id;
                self.daily_locked_amounts.insert((provider_address, day_id), object);
                linked_list.back = day_id;
                linked_list.length += 1;
            } else if day_id < linked_list.head {
                let object = DailyLockedAmount { amount, next_day: linked_list.head };
                linked_list.head = day_id;
                self.daily_locked_amounts.insert((provider_address, day_id), object);
                linked_list.length += 1;
            } else if day_id > linked_list.back {
                self.daily_locked_amounts.get_mut(&(provider_address, linked_list.back)).unwrap().next_day = day_id;
                let object = DailyLockedAmount { amount, next_day: day_id };
                linked_list.back = day_id;
                self.daily_locked_amounts.insert((provider_address, day_id), object);
                linked_list.length += 1;
            } else {
                let mut cur_id: u64 = linked_list.head;
                loop {
                    if day_id == cur_id {
                        self.daily_locked_amounts.get_mut(&(provider_address, day_id)).unwrap().amount += amount;
                        break;
                    } else if day_id < self.daily_locked_amounts.get(&(provider_address, cur_id)).unwrap().next_day {
                        let object = DailyLockedAmount { amount, next_day: self.daily_locked_amounts.get(&(provider_address, cur_id)).unwrap().next_day };
                        self.daily_locked_amounts.get_mut(&(provider_address, cur_id)).unwrap().next_day = day_id;
                        self.daily_locked_amounts.insert((provider_address, day_id), object);
                        linked_list.length += 1;
                        break;
                    }
                    cur_id = self.daily_locked_amounts.get(&(provider_address, cur_id)).unwrap().next_day;
                    if cur_id == linked_list.back {
                        break;
                    }
                }
            }
        }

        /// remove_entry : when a user refunds this function removes its related entry
        /// # arguments:
        /// * provider_address
        /// * day_id : the calculation formula is : (finish date - contract start date) / 86400
        /// * amount
        fn remove_entry(&mut self, provider_address: Account, day_id: u64, amount: u128) {
            self.daily_locked_amounts.get_mut(&(provider_address, day_id)).unwrap().amount -= amount;
        }

        /// process : when providers withdraw this function calculates the amount of money
        /// # arguments:
        /// * provider_address
        /// * day_id : the calculation formula is : (finish date - contract start date) / 86400
        pub fn process(&mut self, provider_address: Account, day_id: u64) -> u128 {
            let linked_list: &mut LinkedList = &mut self.providers.get_mut(&provider_address).unwrap().payment_manager;
            let mut sum: u128 = 0;
            let mut cur_id: u64 = linked_list.head;
            while day_id >= cur_id {
                sum += self.daily_locked_amounts.get(&(provider_address, cur_id)).unwrap().amount;
                cur_id = self.daily_locked_amounts.get(&(provider_address, cur_id)).unwrap().next_day;
                linked_list.length -= 1;
                if cur_id == linked_list.back {
                    break;
                }
            }
            linked_list.head = cur_id;
            sum
        }
    }


    impl Default for LinkedList {
        fn default() -> Self {
            Self::new()
        }
    }

    impl LinkedList {
        pub fn new() -> Self {
            LinkedList::default()
        }
        pub fn default() -> Self {
            Self {
                back: 0,
                head: 0,
                length: 0,
            }
        }
    }
}
