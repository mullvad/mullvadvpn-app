#pragma once

//
// Generic return codes for NSIS plugins
//
enum class NsisStatus
{
	// On stack: error string.
	GENERAL_ERROR = 0,
	// On stack: a string.
	SUCCESS,
};
