// Copyright 2023 STMicroelectronics
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the
//    distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum Error {
	#[error("can't get device-tree name: {0}")]
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
	#[error("can't open {0}: {1}")]
	OpenFailed(String, std::io::Error),

	#[error("can't find entry OF_NAME in {0}")]
	NotFound(String),
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
