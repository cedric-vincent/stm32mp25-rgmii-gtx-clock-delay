use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
	#[error("can't get device name in current device-tree: {0}")]
	GetDtName(#[from] GetDtName),

	#[error("can't get the GPIO connected to the RGMII GTX clock: {0}")]
	GetGpio(#[from] GetGpio),

	#[error("can't get the address of the GPIO connected to the RGMII GTX clock: {0}")]
	GetAddress(#[from] GetAddress),

	#[error("can't get/set the delay of the GPIO connected to the RGMII GTX clock: {0}")]
	GetValue(#[from] GetValue),

	#[error("invalid RGMII clock/data delay")]
	InvalidClockDelay,
}

#[derive(Error, Debug)]
#[error("{0}")]
pub(crate) struct GetDtName(#[from] std::io::Error);

#[derive(Error, Debug)]
#[error("{0}")]
pub(crate) struct GetGpio(#[from] std::io::Error);

#[derive(Error, Debug)]
#[error("{0}")]
pub(crate) struct GetAddress(#[from] std::io::Error);

#[derive(Error, Debug)]
pub(crate) enum GetValue {
	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),

	#[error("OS error: {0}")]
	Os(#[from] nix::Error),
}
