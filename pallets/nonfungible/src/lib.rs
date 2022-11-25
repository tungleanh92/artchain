#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
// pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::storage]
	#[pallet::getter(fn _black_list)]
	pub type BlackList<T: Config> = StorageMap<_, Twox64Concat, u128, bool>;

	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub type OwnerOf<T: Config> = StorageMap<_, Twox64Concat, u128, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub type BalanceOf<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u128>;

	#[pallet::storage]
	#[pallet::getter(fn get_approved)]
	pub type TokenApprovals<T: Config> = StorageMap<_, Twox64Concat, u128, bool>;

	#[pallet::storage]
	#[pallet::getter(fn _is_approved_for_all)]
	// owner id
	pub type OperatorAprrovals<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	#[pallet::storage]
	#[pallet::getter(fn token_uri)]
	pub type TokenUri<T> = StorageMap<_, Twox64Concat, u128, Vec<u8>>;

	#[pallet::storage]
	#[pallet::getter(fn is_listed)]
	pub type Listed<T> = StorageMap<_, Twox64Concat, u128, bool>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Approval { from: T::AccountId, token_id: u128 },
		ApprovalForAll { owner: T::AccountId, approved: bool },
		Transfer { from: T::AccountId, to: T::AccountId, token_id: u128 },
		Mint { to: T::AccountId, token_id: u128 },
		Burn { from: T::AccountId, token_id: u128 },
		List { token_id: u128 },
		Delist { token_id: u128 },
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		NotOwnToken,
		NotOwnerOrNotApprovedForAll,
		ApproveToCaller,
		NotApproved,
		TransferFromIncorrectOwner,
		TokenAlreadyMinted,
		AlreadyListed,
		NotListed
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// marketplace usecase
		#[pallet::weight(0)]
		pub fn approve(origin: OriginFor<T>, token_id: u128) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(OwnerOf::<T>::contains_key(&token_id), Error::<T>::NotOwnToken);
			let owner = OwnerOf::<T>::get(&token_id).unwrap();

			ensure!(
				who == owner || OperatorAprrovals::<T>::contains_key(&who),
				Error::<T>::NotOwnerOrNotApprovedForAll
			);
			Self::_approve(token_id);
			Ok(())
		}

		// marketplace usecase
		#[pallet::weight(0)]
		pub fn set_approve_for_all(
			origin: OriginFor<T>,
			operator: T::AccountId,
			approved: bool,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			ensure!(owner != operator, Error::<T>::ApproveToCaller);
			OperatorAprrovals::<T>::insert(&owner, approved);
			Self::deposit_event(Event::ApprovalForAll { owner, approved });
			Ok(())
		}

		// #[pallet::weight(0)]
		// pub fn transfer_from(
		// 	origin: OriginFor<T>,
		// 	from: T::AccountId,
		// 	to: T::AccountId,
		// 	token_id: u128,
		// ) -> DispatchResult {
		// 	let owner = ensure_signed(origin)?;

		// 	Self::_transfer(from, to, token_id);
		// 	Ok(())
		// }

		#[pallet::weight(0)]
		pub fn mint(origin: OriginFor<T>, token_id: u128, token_uri: Vec<u8>) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			Self::_mint(owner, token_id, token_uri).unwrap();
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn burn(origin: OriginFor<T>, token_id: u128) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			Self::_burn(owner, token_id).unwrap();
			Ok(())
		}

		// auction usecase
		#[pallet::weight(0)]
		pub fn list(origin: OriginFor<T>, token_id: u128) -> DispatchResult {
			ensure_signed(origin)?;
			ensure!(!Listed::<T>::contains_key(&token_id), Error::<T>::AlreadyListed);
			Listed::<T>::insert(&token_id, true);
			Self::deposit_event(Event::List { token_id });
			Ok(())
		}

		// auction usecase
		#[pallet::weight(0)]
		pub fn de_list(origin: OriginFor<T>, token_id: u128) -> DispatchResult {
			ensure_signed(origin)?;
			ensure!(Listed::<T>::contains_key(&token_id), Error::<T>::NotListed);
			Listed::<T>::remove(&token_id);
			Self::deposit_event(Event::Delist { token_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn _approve(token_id: u128) {
			TokenApprovals::<T>::insert(token_id, true);
			let from = OwnerOf::<T>::get(&token_id).unwrap();
			Self::deposit_event(Event::Approval { from, token_id });
		}

		pub fn _transfer(from: T::AccountId, to: T::AccountId, token_id: u128) -> DispatchResult {
			ensure!(
				OperatorAprrovals::<T>::contains_key(&from)
					|| TokenApprovals::<T>::contains_key(&token_id),
				Error::<T>::NotApproved
			);
			let owner = OwnerOf::<T>::get(&token_id).unwrap();

			ensure!(from == owner, Error::<T>::TransferFromIncorrectOwner);

			let balance_from = BalanceOf::<T>::get(&from);
			BalanceOf::<T>::insert(&from, balance_from.unwrap() - 1);

			let to_has_balance = BalanceOf::<T>::contains_key(&to);
			if to_has_balance == true {
				let balance_to = BalanceOf::<T>::get(&to);
				BalanceOf::<T>::insert(&to, balance_to.unwrap() + 1);
			} else {
				BalanceOf::<T>::insert(&to, 1);
			}
			OwnerOf::<T>::insert(token_id, &to);
			Self::deposit_event(Event::Transfer { from, to, token_id });

			Ok(())
		}

		pub fn _mint(to: T::AccountId, token_id: u128, token_uri: Vec<u8>) -> DispatchResult {
			ensure!(OwnerOf::<T>::contains_key(&token_id), Error::<T>::TokenAlreadyMinted);
			let to_has_balance = BalanceOf::<T>::contains_key(&to);
			if to_has_balance == true {
				let balance_to = BalanceOf::<T>::get(&to);
				BalanceOf::<T>::insert(&to, balance_to.unwrap() + 1);
			} else {
				BalanceOf::<T>::insert(&to, 1);
			}
			OwnerOf::<T>::insert(token_id, &to);
			TokenUri::<T>::insert(token_id, token_uri);
			Self::deposit_event(Event::Mint { to, token_id });
			Ok(())
		}

		pub fn _burn(to: T::AccountId, token_id: u128) -> DispatchResult {
			let to_has_balance = BalanceOf::<T>::contains_key(&to);
			if to_has_balance == true {
				let balance_to = BalanceOf::<T>::get(&to);
				BalanceOf::<T>::insert(&to, balance_to.unwrap() - 1);
			} else {
				BalanceOf::<T>::remove(&to);
			}
			OwnerOf::<T>::remove(token_id);
			TokenUri::<T>::remove(token_id);
			Self::deposit_event(Event::Mint { to, token_id });
			Ok(())
		}
	}
}
