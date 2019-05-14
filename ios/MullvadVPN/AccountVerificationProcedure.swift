//
//  AccountVerificationProcedure.swift
//  MullvadVPN
//
//  Created by pronebird on 14/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import ProcedureKit

class AccountVerificationProcedure: GroupProcedure, InputProcedure, OutputProcedure {
    var input: Pending<String>
    var output: Pending<ProcedureResult<AccountVerification>> = .pending

    init(dispatchQueue underlyingQueue: DispatchQueue? = nil, accountToken: String? = nil) {
        self.input = accountToken.flatMap { .ready($0) } ?? .pending

        // Request account data from the API
        // Wrap the original procedure into IgnoreErrorsProcedure to suppress any networking errors
        // from bubbling upwards.
        let networkRequest = IgnoreErrorsProcedure(MullvadAPI.getAccountData(accountToken: accountToken))

        // Transform the response into AccountVerification
        let transformResponse = TransformProcedure<ProcedureResult<JsonRpcResponse<AccountData>>, AccountVerification> { (procedureResult) -> AccountVerification in
            // Unwrap the result of the network request procedure
            switch procedureResult {
            case .success(let response):
                // Unwrap the JSON RPC response
                switch response.result {
                case .success:
                    // Mark account as verified if the account data was successfuly received
                    return .verified

                case .failure(let serverError):
                    // Mark the account as invalid if the server returned an error along with
                    // the JSON RPC response
                    return .invalid(serverError)
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
