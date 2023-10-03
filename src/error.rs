use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
	#[error("{0}")]
	DTGetName(#[from] DTGetName),

	#[error("can't get the GPIO connected to the RGMII GTX clock: {0}")]
	GetGpio(#[from] GetGpio),

	#[error("can't get the address of the GPIO connected to the RGMII GTX clock: {0}")]
	GetAddress(#[from] GetAddress),

	#[error("can't memory-map the address of the GPIO connected to the RGMII GTX clock: {0}")]
	MmapValue(#[from] MmapValue),

	#[error("invalid RGMII clock/data delay")]
	InvalidClockDelay,

	#[error("can't download: {0}")]
	Download(#[from] curl::Error),
}

#[derive(Error, Debug)]
pub(crate) enum DTGetName {
	#[error("Can't open {0}: {1}")]
	OpenFailed(String, std::io::Error),

	#[error("Can't find device-tree name of {0} in {1}")]
	NotFound(String, String),
}

#[derive(Error, Debug)]
#[error("{0}")]
pub(crate) struct GetGpio(#[from] std::io::Error);

#[derive(Error, Debug)]
#[error("{0}")]
pub(crate) struct GetAddress(#[from] std::io::Error);

#[derive(Error, Debug)]
pub(crate) enum MmapValue {
	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),

	#[error("OS error: {0}")]
	Os(#[from] nix::Error),
}
