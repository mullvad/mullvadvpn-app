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
