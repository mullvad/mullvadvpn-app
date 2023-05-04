//
//  NSFileCoordinator+Extensions.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2023-05-11.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension NSFileCoordinator {
    public func coordinate<R>(
        readingItemAt itemURL: URL,
        options: ReadingOptions = [],
        accessor: (URL) throws -> R
    ) throws -> R {
        var error: NSError?
        var result: Result<R, Error> = .failure(CocoaError(.fileReadUnknown))

        coordinate(readingItemAt: itemURL, options: options, error: &error) { url in
            result = Result { try accessor(url) }
        }

        if let error {
            throw error
        }

        return try result.get()
    }

    public func coordinate(
        writingItemAt itemURL: URL,
        options: WritingOptions = [],
        accessor: (URL) throws -> Void
    ) throws {
        var error: NSError?
        var accessorError: Error?

        coordinate(writingItemAt: itemURL, options: options, error: &error) { url in
            do {
                try accessor(url)
            } catch {
                accessorError = error
            }
        }

        if let e = error ?? accessorError {
            throw e
        }
    }
}
