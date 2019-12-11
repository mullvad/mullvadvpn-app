#pragma once

//
// Generic return codes for NSIS plugins
//
//
// Returned on the stack:
//
//   GENERAL_ERROR:  Most functions return an error message.
//   SUCCESS:        Most functions return an empty string.
//
// NOTE: While this is generally true, some functions only
//       push a status code to the stack.
//

enum NsisStatus
{
	GENERAL_ERROR = 0,
	SUCCESS,
};
