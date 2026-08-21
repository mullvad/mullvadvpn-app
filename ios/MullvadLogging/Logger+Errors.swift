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
import Logging
import MullvadTypes

extension Logger {
    public func error(
        error: some Error,
        message: @autoclosure () -> String? = nil,
        metadata: @autoclosure () -> Logger.Metadata? = nil,
        source: @autoclosure () -> String? = nil,
        file: String = #file,
        function: String = #function,
        line: UInt = #line
    ) {
        var lines = [String]()
        var errors = [Error]()

        if let prefixMessage = message() {
            lines.append(prefixMessage)
            errors.append(error)
        } else {
            lines.append(error.logFormatError())
        }

        errors.append(contentsOf: error.underlyingErrorChain)

        for error in errors {
            lines.append("Caused by: \(error.logFormatError())")
        }

        log(
            level: .error,
            Message(stringLiteral: lines.joined(separator: "\n")),
            metadata: metadata(),
            source: source(),
            file: file,
            function: function,
            line: line
        )
    }
}
