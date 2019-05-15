//
//  AccountVerificationProcedure.swift
//  MullvadVPN
//
//  Created by pronebird on 14/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import ProcedureKit

/// Account verification result
enum AccountVerification {
    /// The app should attempt to verify the account token at some point later because the network
    /// may not be available at this time.
    case deferred(Error)

    /// The app successfully verified the account token with the server
    case verified(Date)

    // Invalid token
    case invalid(Error)
}

private let kAccountDoesNotExistErrorCode = -200

/// The procedure that implements account verification by sending the account expiry request to the
/// Mullvad API. This procedure is non-fallable so even in the case of network issues it will set
/// the output.
class AccountVerificationProcedure: GroupProcedure, InputProcedure, OutputProcedure {
    var input: Pending<String>
    var output: Pending<ProcedureResult<AccountVerification>> = .pending

    init(dispatchQueue underlyingQueue: DispatchQueue? = nil, accountToken: String? = nil) {
        self.input = accountToken.flatMap { .ready($0) } ?? .pending

        // Request account data from the API
        // Wrap the original procedure into IgnoreErrorsProcedure to suppress any networking errors
        // from bubbling upwards.
        let networkRequest = IgnoreErrorsProcedure(MullvadAPI.getAccountExpiry(accountToken: accountToken))

        // Transform the response into AccountVerification
        let transformResponse = TransformProcedure<ProcedureResult<JsonRpcResponse<Date>>, AccountVerification> { (procedureResult) -> AccountVerification in
            // Unwrap the result of the network request procedure
            switch procedureResult {
            case .success(let response):
                // Unwrap the JSON RPC response
                switch response.result {
                case .success(let expiryDate):
                    return .verified(expiryDate)

                case .failure(let serverError):
                    if serverError.code == kAccountDoesNotExistErrorCode {
                        return .invalid(serverError)
                    } else {
                        return .deferred(serverError)
                    }
                }
            case .failure(let networkError):
                // Check back later in case of network issues
                return .deferred(networkError)
            }
        }

        // Normally injectResult takes care of the
        transformResponse.inject(dependency: networkRequest) {
            (transformProcedure, networkProcedure, _) in
            transformProcedure.input = networkProcedure.output
        }

        super.init(dispatchQueue: underlyingQueue, operations: [networkRequest, transformResponse])

        // Wire up the output of the transformResponse to the output of the entire group
        bind(from: transformResponse)

        // Copy the input of the group procedure to the input of the starting procedure
        addWillExecuteBlockObserver { [weak networkRequest] (groupProcedure, _) in
            networkRequest?.input = groupProcedure.input
        }
    }
}
