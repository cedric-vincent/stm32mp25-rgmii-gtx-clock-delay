use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
	#[error("{0}")]
	GetDtName(#[from] GetDtName),

	#[error("{0}")]
	GetGpio(#[from] GetGpio),

	#[error("{0}")]
	GetAddress(#[from] GetAddress),
}

#[derive(Error, Debug)]
#[error("can't get the address of the GPIO connected to the RGMII GTX clock: {0}")]
pub(crate) struct GetAddress(#[from] std::io::Error);

#[derive(Error, Debug)]
#[error("can't get the GPIO connected to the RGMII GTX clock: {0}")]
pub(crate) struct GetGpio(#[from] std::io::Error);

#[derive(Error, Debug)]
#[error("can't get device name in current device-tree: {0}")]
pub(crate) struct GetDtName(#[from] std::io::Error);

