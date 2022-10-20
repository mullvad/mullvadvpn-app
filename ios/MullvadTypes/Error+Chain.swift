//
//  Error+Chain.swift
//  MullvadTypes
//
//  Created by pronebird on 23/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

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
        var message = ""

        let description = (self as? CustomErrorDescriptionProtocol)?
            .customErrorDescription ?? localizedDescription

        message += "\(description) (domain = \(nsError.domain), code = \(nsError.code))"

        return message
    }

    private func getUnderlyingError() -> Error? {
        if let wrappingError = self as? WrappingError {
            return wrappingError.underlyingError
        } else {
            return (self as NSError).userInfo[NSUnderlyingErrorKey] as? Error
        }
    }
}
