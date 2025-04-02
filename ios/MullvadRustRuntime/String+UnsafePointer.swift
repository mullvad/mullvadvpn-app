//
//  String+UnsafePointer.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-04-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension String {
    /// Converts the `String` to an `UnsafePointer<CChar>` (null-terminated C string).
    /// The pointer is valid as long as the string is alive.
    func toCStringPointer() -> UnsafePointer<CChar>? {
        // Ensure the string is converted to a null-terminated C string
        var pointer: UnsafePointer<CChar>?
        self.withCString { cString in
            pointer = cString
        }
        return pointer
    }
}
