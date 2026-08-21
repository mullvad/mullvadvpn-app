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

extension Error {
    /// Returns a flat list of errors by unrolling the underlying error chain.
    public var underlyingErrorChain: [Error] {
        var errors: [Error] = []
        var currentError: Error? = self as Error

        while let underlyingError = currentError?.getUnderlyingError() {
            currentError = underlyingError
            errors.append(underlyingError)
        }

        return errors
    }

    public func logFormatError() -> String {
        let nsError = self as NSError
        let description =
            (self as? CustomErrorDescriptionProtocol)?
            .customErrorDescription ?? nsError.description

        return description
    }

    private func getUnderlyingError() -> Error? {
        if let wrappingError = self as? WrappingError {
            return wrappingError.underlyingError
        } else {
            return (self as NSError).userInfo[NSUnderlyingErrorKey] as? Error
        }
    }
}
