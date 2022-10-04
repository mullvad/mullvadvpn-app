//
//  RequestModel.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct EncodableModel: Codable {
    let url: URL?
    let method: String?
    let serializedParameters: Data?

    func encode() throws -> Data {
        try JSONEncoder().encode(self)
    }
}

extension EncodableModel {
    init(urlRequest: URLRequest) {
        url = urlRequest.url
        method = urlRequest.httpMethod
        serializedParameters = urlRequest.httpBody
    }
}

struct DecodableModel: Codable {
    let data: Data?
    let response: String?
    let error: String?
}
