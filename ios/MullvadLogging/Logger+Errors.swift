//
//  Logger+Errors.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import MullvadTypes

extension Logger {
    public func error<T: Error>(
        error: T,
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
