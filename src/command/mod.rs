pub mod all;
pub mod args;

use serenity::all::{Command, CreateCommand};

#[macro_export]
macro_rules! arg {
	(Int, $name:literal, $desc:literal, $required:literal, $min:expr, $max:expr) => {
		IntArg {
			base: BaseArg {
				name: $name,
				description: $desc,
				required: $required,
			},
			min: $min,
			max: $max,
		}
	};
	(User, $name:literal, $desc:literal, $required:literal) => {
		UserArg {
			base: BaseArg {
				name: $name,
				description: $desc,
				required: $required,
			},
		}
	};
	(String, $name:literal, $desc:literal, $required:literal, $choices:expr) => {
		StringArg {
			base: BaseArg {
				name: $name,
				description: $desc,
				required: $required,
			},
			choices: $choices,
		}
	};
}

pub trait IntoCommand: PartialEq<Command> {
	const NAME: &'static str;
	fn into_command() -> CreateCommand;
}

#[doc(hidden)]
macro_rules! command_inner {
	($type_name:ident, $name:literal, $desc:literal, $dm_permission:literal, $permissions:expr, $args:expr) => {
		pub struct $type_name;

		impl PartialEq<serenity::all::Command> for $type_name {
			fn eq(&self, other: &serenity::all::Command) -> bool {
				other.dm_permission == Some($dm_permission)
					&& other.default_member_permissions == $permissions
					&& other.name == $name
					&& other.description == $desc
					&& {
						if other.options.len() != $args.len() {
							false
						} else {
							let old_args = other
								.options
								.iter()
								.map(|arg| (arg.name.to_owned(), arg))
								.collect::<HashMap<_, _>>();
							$args.iter().all(|new_arg: &&dyn IntoCommandArg| {
								old_args
									.get(new_arg.name())
									.map(|old_arg| new_arg == old_arg)
									.unwrap_or(false)
							})
						}
					}
			}
		}

		impl $crate::command::IntoCommand for $type_name {
			const NAME: &'static str = $name;

			fn into_command() -> serenity::all::CreateCommand {
				let mut cmd = serenity::all::CreateCommand::new($name)
					.description($desc)
					.dm_permission($dm_permission)
					.set_options(
						$args
							.iter()
							.map(|arg: &&dyn IntoCommandArg| arg.to_arg())
							.collect(),
					);
				if let Some(permissions) = $permissions {
					cmd = cmd.default_member_permissions(permissions);
				}
				cmd
			}
		}

		impl core::fmt::Display for $type_name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				f.write_str($name)
			}
		}
	};
}
pub(in crate::command) use command_inner;

macro_rules! command {
	($type_name:ident, $name:literal, $desc:literal, $dm_permission:literal, $permissions:expr, [$($args:expr), *]) => {
		$crate::command::command_inner!($type_name, $name, $desc, $dm_permission, Some($permissions), &[$(&$args as &dyn IntoCommandArg), *]);
	};
	($type_name:ident, $name:literal, $desc:literal, $dm_permission:literal, [$($args:expr), *]) => {
		$crate::command::command_inner!($type_name, $name, $desc, $dm_permission, None::<Permissions>, &[$(&$args as &dyn IntoCommandArg), *]);
	};
	($type_name:ident, $name:literal, $desc:literal, $dm_permission:literal, $permissions:expr) => {
		$crate::command::command_inner!($type_name, $name, $desc, $dm_permission, Some($permissions), &[] as &[&dyn IntoCommandArg]);
	};
	($type_name:ident, $name:literal, $desc:literal, [$($args:expr), *]) => {
		$crate::command::command_inner!($type_name, $name, $desc, true, None::<Permissions>, &[$(&$args as &dyn IntoCommandArg), *] as &[&dyn IntoCommandArg]);
	};
	($type_name:ident, $name:literal, $desc:literal) => {
		$crate::command::command_inner!($type_name, $name, $desc, true, None::<Permissions>, &[] as &[&dyn IntoCommandArg]);
	};
}
pub(crate) use command;
