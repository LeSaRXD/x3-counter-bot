pub mod all;

use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, Permissions};

pub trait IntoCommandArg {
	fn to_arg(&self) -> CreateCommandOption;
}
pub struct IntArg {
	pub name: &'static str,
	pub description: &'static str,
	pub required: bool,
	pub min: Option<u64>,
	pub max: Option<u64>,
}

impl IntoCommandArg for IntArg {
	fn to_arg(&self) -> CreateCommandOption {
		let mut option =
			CreateCommandOption::new(CommandOptionType::Integer, self.name, self.description)
				.required(self.required);
		if let Some(min) = self.min {
			option = option.min_int_value(min);
		}
		if let Some(max) = self.max {
			option = option.max_int_value(max);
		}
		option
	}
}

#[macro_export]
macro_rules! arg {
	(Int, $name:literal, $desc:literal, $required:literal, $min:expr, $max:expr) => {
		IntArg {
			name: $name,
			description: $desc,
			required: $required,
			min: $min,
			max: $max,
		}
	};
}

pub trait IntoCommand {
	const NAME: &'static str;
	const DESCRIPTION: &'static str;
	const DM_PERMISSION: bool;
	const PERMISSIONS: Permissions;
	const ARGS: &'static [&'static dyn IntoCommandArg];

	fn into_command() -> CreateCommand {
		CreateCommand::new(Self::NAME)
			.description(Self::DESCRIPTION)
			.dm_permission(Self::DM_PERMISSION)
			.default_member_permissions(Self::PERMISSIONS)
			.set_options(Self::ARGS.iter().map(|arg| arg.to_arg()).collect())
	}
}

#[macro_export]
macro_rules! command {
	($type_name: ident, $name:literal, $desc:literal, $dm_permission:literal, $permissions:expr, [$($args:expr), *]) => {
		pub struct $type_name;
		impl command::IntoCommand for $type_name {
			const NAME: &'static str = $name;
			const DESCRIPTION: &'static str = $desc;
			const DM_PERMISSION: bool = $dm_permission;
			const PERMISSIONS: Permissions = $permissions;
			const ARGS: &'static [&'static dyn command::IntoCommandArg] = &[$(&$args), *];
		}
		impl core::fmt::Display for $type_name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				f.write_str($name)
			}
		}
	};
	($type_name: ident, $name:literal, $desc:literal, $dm_permission:literal, [$($args:expr), *]) => {
		command!($type_name, $name, $desc, $dm_permission, Permissions::empty(), [$($args), *]);
	};
	($type_name: ident, $name:literal, $desc:literal, $dm_permission:literal, $permissions:expr) => {
		command!($type_name, $name, $desc, $dm_permission, $permissions, []);
	};
	($type_name: ident, $name:literal, $desc:literal, [$($args:expr), *]) => {
		command!($type_name, $name, $desc, true, Permissions::empty(), [$($args), *]);
	};
	($type_name: ident, $name:literal, $desc:literal) => {
		command!($type_name, $name, $desc, true, Permissions::empty(), []);
	};
}
