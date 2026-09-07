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
import MullvadREST
import XCTest

class PartnerAPIClient {
    let baseURL = URL(string: "https://partner.stagemole.eu/v1/")!

    lazy var accessToken: String = {
        guard let token = Bundle(for: BaseUITestCase.self).infoDictionary?["PartnerApiToken"] as? String else {
            fatalError("Failed to retrieve partner API token from config")
        }
        return token
    }()

    /// Add time to an account
    /// - Parameters:
    ///   - accountNumber: Account number
    ///   - days: Number of days to add. Needs to be between 1 and 31.
    func addTime(accountNumber: String, days: Int) async -> Date {
        let jsonResponse = await sendRequest(
            method: "POST",
            endpoint: "accounts/\(accountNumber)/extend",
            jsonObject: ["days": "\(days)"]
        )

        guard let newExpiryString = jsonResponse["new_expiry"] as? String else {
            XCTFail("Failed to read new account expiry from response")
            return Date()
        }

        let dateFormatter = ISO8601DateFormatter()
        guard let newExpiryDate = dateFormatter.date(from: newExpiryString) else {
            XCTFail("Failed to create Date object from date string")
            return Date()
        }

        return newExpiryDate
    }

    func createAccount() async -> String {
        let jsonResponse = await sendRequest(method: "POST", endpoint: "accounts", jsonObject: nil)

        guard let accountNumber = jsonResponse["id"] as? String else {
            XCTFail("Failed to read created account number")
            return String()
        }

        return accountNumber
    }

    func deleteAccount(accountNumber: String) async {
        _ = await sendRequest(method: "DELETE", endpoint: "accounts/\(accountNumber)", jsonObject: nil)
    }

    private func sendRequest(method: String, endpoint: String, jsonObject: [String: Any]?) async -> [String: Any] {
        var request = URLRequest(url: baseURL.appendingPathComponent(endpoint))
        request.setValue("Basic \(accessToken)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpMethod = method

        if let jsonObject {
            do {
                request.httpBody = try JSONSerialization.data(withJSONObject: jsonObject)
            } catch {
                XCTFail("Failed to serialize JSON object")
                return [:]
            }
        }

        let retryStrategy = REST.RetryStrategy.default
        let delayIterator = retryStrategy.makeDelayIterator()

        for attempt in 0...retryStrategy.maxRetryCount {
            if let result = try? await attemptRequest(request) {
                return result
            }

            guard attempt < retryStrategy.maxRetryCount else { break }

            if let delay = delayIterator.next() {
                try? await Task.sleep(for: delay)
            }
        }

        XCTFail("\(method) \(endpoint) failed after \(retryStrategy.maxRetryCount + 1) attempt(s)")
        return [:]
    }

    private func attemptRequest(_ request: URLRequest) async throws -> [String: Any]? {
        let (data, response) = try await URLSession.shared.data(for: request)

        guard
            let response = response as? HTTPURLResponse,
            200...204 ~= response.statusCode
        else {
            print("Response error: \(response.description)")
            throw URLError(.badServerResponse)
        }

        return if data.isEmpty {
            [:]
        } else {
            try JSONSerialization.jsonObject(with: data) as? [String: Any]
        }
    }
}
