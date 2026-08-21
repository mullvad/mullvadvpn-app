// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

extension String {
    // Ensure the string is converted to a null-terminated C string
    // UnsafePointer provides no automated memory management or alignment guarantees.
    // The caller is responsible to manage the memory
    func toCStringPointer() -> UnsafePointer<CChar>? {
        // Convert the Swift string to a null-terminated UTF-8 C string
        guard let cString = cString(using: .utf8) else { return nil }

        // Allocate memory for characters + null terminator
        let pointer = UnsafeMutablePointer<CChar>.allocate(capacity: cString.count)

        // Copy the characters (including the null terminator)
        pointer.initialize(from: cString, count: cString.count)

        return UnsafePointer(pointer)
    }
}
