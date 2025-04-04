//
//  String+UnsafePointer.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation
extension String {
    /// Converts the `String` to an `UnsafePointer<UInt8>`. The memory allocated for the pointer
    /// is not null-terminated, so the caller must ensure that they manage the memory properly.
    func toUnsafePointer() -> (pointer: UnsafePointer<UInt8>?, size: UInt)? {
        guard let data = self.data(using: .utf8) else { return nil }

        let buffer = UnsafeMutablePointer<UInt8>.allocate(capacity: data.count)
        data.copyBytes(to: buffer, count: data.count)

        return (UnsafePointer(buffer), UInt(data.count))
    }
}
