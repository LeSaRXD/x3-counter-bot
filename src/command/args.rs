use serenity::all::{CommandOption, CommandOptionType, CreateCommandOption};

pub trait IntoCommandArg: PartialEq<CommandOption> {
	fn name(&self) -> &str;
	fn to_arg(&self) -> CreateCommandOption;
}
#[derive(Debug, Clone)]
pub struct IntArg {
	pub name: &'static str,
	pub description: &'static str,
	pub required: bool,
	pub min: Option<u64>,
	pub max: Option<u64>,
}

impl PartialEq<CommandOption> for IntArg {
	fn eq(&self, other: &CommandOption) -> bool {
		other.kind == CommandOptionType::Integer
			&& other.required == self.required
			&& other.min_value.as_ref().and_then(|v| v.as_u64()) == self.min
			&& other.max_value.as_ref().and_then(|v| v.as_u64()) == self.max
			&& other.name == self.name
			&& other.description == self.description
	}
}

impl IntoCommandArg for IntArg {
	fn name(&self) -> &str {
		self.name
	}

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
