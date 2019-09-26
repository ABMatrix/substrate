// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Storage vec abstraction on top of runtime storage, using hashed keys.

use rstd::{prelude::*, borrow::Borrow};
use codec::{Codec, KeyedVec};
use runtime_io::{self, twox_128};
use super::hashed;

/// A trait to conveniently store a vector of storable data.
///
/// It uses twox_128 hasher. Final keys in trie are `twox_128(concatenation(PREFIX,count))`
pub trait StorageVec {
	type Item: Default + Sized + Codec;
	const PREFIX: &'static [u8];

	/// Get the current set of items.
	fn items() -> Vec<Self::Item> {
		(0..Self::count()).into_iter().map(Self::item).collect()
	}

	/// Set the current set of items.
	fn set_items<I, T>(items: I)
		where
			I: IntoIterator<Item=T>,
			T: Borrow<Self::Item>,
	{
		let mut count: u32 = 0;

		for i in items.into_iter() {
			hashed::put(&twox_128, &count.to_keyed_vec(Self::PREFIX), i.borrow());
			count = count.checked_add(1).expect("exceeded runtime storage capacity");
		}

		Self::set_count(count);
	}

	/// Push an item.
	fn push(item: &Self::Item) {
		let len = Self::count();
		hashed::put(&twox_128, &len.to_keyed_vec(Self::PREFIX), item);
		Self::set_count(len + 1);
	}

	fn set_item(index: u32, item: &Self::Item) {
		if index < Self::count() {
			hashed::put(&twox_128, &index.to_keyed_vec(Self::PREFIX), item);
		}
	}

	fn clear_item(index: u32) {
		if index < Self::count() {
			hashed::kill(&twox_128, &index.to_keyed_vec(Self::PREFIX));
		}
	}

	fn item(index: u32) -> Self::Item {
		hashed::get_or_default(&twox_128, &index.to_keyed_vec(Self::PREFIX))
	}

	fn set_count(count: u32) {
		(count..Self::count()).for_each(Self::clear_item);
		hashed::put(&twox_128, &b"len".to_keyed_vec(Self::PREFIX), &count);
	}

	fn count() -> u32 {
		hashed::get_or_default(&twox_128, &b"len".to_keyed_vec(Self::PREFIX))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use runtime_io::{twox_128, TestExternalities, with_externalities};

	#[test]
	fn integers_can_be_stored() {
		let mut t = TestExternalities::default();
		with_externalities(&mut t, || {
			let x = 69u32;
			hashed::put(&twox_128, b":test", &x);
			let y: u32 = hashed::get(&twox_128, b":test").unwrap();
			assert_eq!(x, y);
		});
		with_externalities(&mut t, || {
			let x = 69426942i64;
			hashed::put(&twox_128, b":test", &x);
			let y: i64 = hashed::get(&twox_128, b":test").unwrap();
			assert_eq!(x, y);
		});
	}

	#[test]
	fn bools_can_be_stored() {
		let mut t = TestExternalities::default();
		with_externalities(&mut t, || {
			let x = true;
			hashed::put(&twox_128, b":test", &x);
			let y: bool = hashed::get(&twox_128, b":test").unwrap();
			assert_eq!(x, y);
		});

		with_externalities(&mut t, || {
			let x = false;
			hashed::put(&twox_128, b":test", &x);
			let y: bool = hashed::get(&twox_128, b":test").unwrap();
			assert_eq!(x, y);
		});
	}

	#[test]
	fn vecs_can_be_retrieved() {
		let mut t = TestExternalities::default();
		with_externalities(&mut t, || {
			runtime_io::set_storage(&twox_128(b":test"), b"\x2cHello world");
			let x = b"Hello world".to_vec();
			let y = hashed::get::<Vec<u8>, _, _>(&twox_128, b":test").unwrap();
			assert_eq!(x, y);
		});
	}

	#[test]
	fn vecs_can_be_stored() {
		let mut t = TestExternalities::default();
		let x = b"Hello world".to_vec();

		with_externalities(&mut t, || {
			hashed::put(&twox_128, b":test", &x);
		});

		with_externalities(&mut t, || {
			let y: Vec<u8> = hashed::get(&twox_128, b":test").unwrap();
			assert_eq!(x, y);
		});
	}
}
