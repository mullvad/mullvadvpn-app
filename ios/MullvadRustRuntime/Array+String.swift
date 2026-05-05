//
//  Array+String.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-30.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import Foundation

extension Array where Element == String {
    func withCStringArray<T>(
        _ body: (UnsafePointer<UnsafePointer<CChar>?>?, UInt) -> T
    ) -> T {
        let mutablePtrs = self.map { strdup($0)! }

        defer {
            for ptr in mutablePtrs {
                free(ptr)
            }
        }

        // Wrap each pointer as optional
        let optionalPtrs: [UnsafePointer<CChar>?] = mutablePtrs.map {
            UnsafePointer($0)
        }

        return optionalPtrs.withUnsafeBufferPointer { buffer in
            body(buffer.baseAddress, UInt(buffer.count))
        }
    }
}
