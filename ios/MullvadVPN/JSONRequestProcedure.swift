//
//  JSONRequestProcedure.swift
//  MullvadVPN
//
//  Created by pronebird on 14/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import ProcedureKit

final class JSONRequestProcedure<Input, Output: Decodable>: GroupProcedure, InputProcedure, OutputProcedure {

    typealias URLRequestBuilder = (Input) throws -> URLRequest

    var input: Pending<Input>
    var output: Pending<ProcedureResult<Output>> = .pending

    init(dispatchQueue underlyingQueue: DispatchQueue? = nil, input: Input? = nil, requestBuilder: @escaping URLRequestBuilder) {
        self.input = input.flatMap { .ready($0) } ?? .pending

        let createRequest = TransformProcedure { try requestBuilder($0) }

        let networkRequest = NetworkProcedure {
            NetworkDataProcedure(session: URLSession.shared)
            }.injectResult(from: createRequest)

        let payloadParsing = DecodeJSONProcedure<Output>(
            dateDecodingStrategy: .iso8601,
            keyDecodingStrategy: .convertFromSnakeCase
            ).injectPayload(fromNetwork: networkRequest)

        super.init(dispatchQueue: underlyingQueue, operations: [createRequest, networkRequest, payloadParsing])

        bind(from: payloadParsing)

        addWillExecuteBlockObserver { (procedure, _) in
            createRequest.input = procedure.input
        }
    }
}

extension JSONRequestProcedure where Input == Void {
    convenience init(requestBuilder: @escaping URLRequestBuilder) {
        self.init(input: (), requestBuilder: requestBuilder)
    }
}
