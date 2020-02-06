//
//  ReplaceNilWithError.swift
//  MullvadVPN
//
//  Created by pronebird on 19/11/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation

extension Publisher {

    /// Replace nil elements with the provided error
    func replaceNil<T>(with error: Failure) -> Publishers.FlatMap<Result<T, Failure>.Publisher, Self>
        where Output == T?, Failure: Error {
        return flatMap { (output: T?) -> Result<T, Failure>.Publisher in
            let result: Result<T, Failure> = output.flatMap { .success($0) } ?? .failure(error)

            return result.publisher
        }
    }

}
