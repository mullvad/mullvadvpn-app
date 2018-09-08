#pragma once

#include <functional>
#include <vector>
#include <string>
#include <cstdint>

bool ConfineOperation
(
	const char *literalOperation,
	std::function<void(const char *, const char **, uint32_t)> errorCallback,
	std::function<void()> operation
);

//
// The returned buffer looks like this:
//
// string pointer 1
// string pointer 2
// string pointer n
// string 1
// string 2
// string n
//
std::vector<uint8_t> CreateRawStringArray(const std::vector<std::string> &arr);
