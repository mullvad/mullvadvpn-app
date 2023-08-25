//
//  TimeServerProxy.swift
//  MullvadRESTTests
//
//  Created by pronebird on 25/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST

/// Simple API proxy used for testing purposes.
final class TimeServerProxy: REST.Proxy<REST.ProxyConfiguration> {
    init(configuration: REST.ProxyConfiguration) {
        super.init(
            name: "TimeServerProxy",
            configuration: configuration,
            requestFactory: REST.RequestFactory.withDefaultAPICredentials(
                pathPrefix: "",
                bodyEncoder: REST.Coding.makeJSONEncoder()
            ),
            responseDecoder: REST.Coding.makeJSONDecoder()
        )
    }

    func getDateTime() -> any RESTRequestExecutor<TimeResponse> {
        let requestHandler = REST.AnyRequestHandler { endpoint in
            return try self.requestFactory.createRequest(endpoint: endpoint, method: .get, pathTemplate: "date-time")
        }
        let responseHandler = REST.defaultResponseHandler(decoding: TimeResponse.self, with: responseDecoder)

        return makeRequestExecutor(
            name: "get-date-time",
            requestHandler: requestHandler,
            responseHandler: responseHandler
        )
    }
}

struct TimeResponse: Codable {
    var dateTime: Date
}

extension REST.ProxyFactory {
    func createTimeServerProxy() -> TimeServerProxy {
        return TimeServerProxy(configuration: configuration)
    }
}
