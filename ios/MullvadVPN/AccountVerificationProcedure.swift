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
    case invalid
}

/// The error code returned by the API when it cannot find the given account token
private let kAccountDoesNotExistErrorCode = -200

/// The procedure that implements account verification by sending the account expiry request to the
/// Mullvad API. This procedure is non-fallable so even in the case of network issues it will set
/// the output and return no errors.
class AccountVerificationProcedure: GroupProcedure, InputProcedure, OutputProcedure {
    var input: Pending<String>
    var output: Pending<ProcedureResult<AccountVerification>> = .pending

    init(dispatchQueue underlyingQueue: DispatchQueue? = nil, accountToken: String? = nil) {
        self.input = accountToken.flatMap { .ready($0) } ?? .pending

        // Request account data from the API
        let networkRequest = MullvadAPI.getAccountExpiry(accountToken: accountToken)

        super.init(dispatchQueue: underlyingQueue, operations: [
            // Wrap the network request into the ignoreErrorsProcedure to make sure that any network
            // or JSON decoding errors do not get propagates. These errors will be returned along
            // with the AccountVerification via the output.
            IgnoreErrorsProcedure(dispatchQueue: underlyingQueue, operation: networkRequest)
        ])

        // Copy the input of the group procedure to the input of the starting procedure
        addWillExecuteBlockObserver { [weak networkRequest] (groupProcedure, _) in
            networkRequest?.input = groupProcedure.input
        }

        networkRequest.addWillFinishBlockObserver { [weak self] (networkRequest, error, _) in
            guard let self = self else { return }

            // Obtain the network error or the procedure result
            guard let procedureResult = error.flatMap({ .failure($0) })
                    ?? networkRequest.output.value else { return }

            // Do not set the output if the network request was cancelled
            if !networkRequest.isCancelled {
                self.output = .ready(.success(self.mapResult(procedureResult)))
            }
        }
    }

    private func mapResult(_ procedureResult: ProcedureResult<JsonRpcResponse<Date>>) -> Output {
        // Unwrap the result of the network request procedure
        switch procedureResult {
        case .success(let response):
            // Unwrap the JSON RPC response
            switch response.result {
            case .success(let expiryDate):
                return .verified(expiryDate)

            case .failure(let serverError):
                if serverError.code == kAccountDoesNotExistErrorCode {
                    return .invalid
                } else {
                    return .deferred(serverError)
                }
            }
        case .failure(let networkError):
            // Check back later in case of network issues
            return .deferred(networkError)
        }
    }
}
