use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
	#[error("{0}")]
	FindDtName(#[from] FindDtName),

	#[error("{0}")]
	FindGpio(#[from] FindGpio),

	#[error("{0}")]
	FindAddress(#[from] FindAddress),
}

#[derive(Error, Debug)]
#[error("can't find the address of the GPIO connected to the RGMII GTX clock: {0}")]
pub(crate) struct FindAddress(#[from] std::io::Error);

#[derive(Error, Debug)]
#[error("can't find the GPIO connected to the RGMII GTX clock: {0}")]
pub(crate) struct FindGpio(#[from] std::io::Error);

#[derive(Error, Debug)]
#[error("can't find device name in current device-tree: {0}")]
pub(crate) struct FindDtName(#[from] std::io::Error);

