use std::error::Error;

pub trait ErrMessage<T> {
	fn with_msg(self, msg: &str) -> Result<T, String>;
}

impl<T, E: Error> ErrMessage<T> for Result<T, E> {
	fn with_msg(self, msg: &str) -> Result<T, String> {
		match self {
			Ok(a) => Ok(a),
			Err(e) => Err(format!("{}: {}", msg, e)),
		}
	}
}
