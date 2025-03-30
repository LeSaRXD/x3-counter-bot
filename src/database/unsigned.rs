use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PsqlU<N>(pub N);

pub type PsqlU64 = PsqlU<u64>;
impl From<u64> for PsqlU<u64> {
	fn from(value: u64) -> Self {
		Self(value)
	}
}
impl From<PsqlU<u64>> for u64 {
	fn from(value: PsqlU<u64>) -> Self {
		value.0
	}
}
impl From<i64> for PsqlU<u64> {
	fn from(value: i64) -> Self {
		Self(value as u64)
	}
}
impl From<PsqlU<u64>> for i64 {
	fn from(value: PsqlU<u64>) -> Self {
		value.0 as i64
	}
}
impl Display for PsqlU<u64> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

pub type PsqlU32 = PsqlU<u32>;
impl From<u32> for PsqlU<u32> {
	fn from(value: u32) -> Self {
		Self(value)
	}
}
impl From<PsqlU<u32>> for u32 {
	fn from(value: PsqlU<u32>) -> Self {
		value.0
	}
}
impl From<i32> for PsqlU<u32> {
	fn from(value: i32) -> Self {
		Self(value as u32)
	}
}
impl From<PsqlU<u32>> for i32 {
	fn from(value: PsqlU<u32>) -> Self {
		value.0 as i32
	}
}
impl Display for PsqlU<u32> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}
