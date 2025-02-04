//
//  PartnerAPIClient.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-04-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
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
    func addTime(accountNumber: String, days: Int) -> Date {
        let jsonResponse = sendRequest(
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

    func createAccount() -> String {
        let jsonResponse = sendRequest(method: "POST", endpoint: "accounts", jsonObject: nil)

        guard let accountNumber = jsonResponse["id"] as? String else {
            XCTFail("Failed to read created account number")
            return String()
        }

        return accountNumber
    }

    func deleteAccount(accountNumber: String) {
        _ = sendRequest(method: "DELETE", endpoint: "accounts/\(accountNumber)", jsonObject: nil)
    }

    private func sendRequest(method: String, endpoint: String, jsonObject: [String: Any]?) -> [String: Any] {
        let url = baseURL.appendingPathComponent(endpoint)
        var request = URLRequest(url: url)
        request.httpMethod = method
        request.setValue("Basic \(accessToken)", forHTTPHeaderField: "Authorization")

        var jsonResponse: [String: Any] = [:]

        do {
            if let jsonObject = jsonObject {
                request.setValue("application/json", forHTTPHeaderField: "Content-Type")
                request.httpBody = try JSONSerialization.data(withJSONObject: jsonObject, options: [])
            }
        } catch {
            XCTFail("Failed to serialize JSON object")
            return [:]
        }

        let completionHandlerInvokedExpectation = XCTestExpectation(
            description: "Completion handler for the request is invoked"
        )

        var requestError: Error?

        let task = URLSession.shared.dataTask(with: request) { data, response, error in
            requestError = error

            guard let data = data,
                  let response = response as? HTTPURLResponse,
                  error == nil else {
                XCTFail("Error: \(error?.localizedDescription ?? "Unknown error")")
                completionHandlerInvokedExpectation.fulfill()
                return
            }

            if 200 ... 204 ~= response.statusCode {
                print("Request successful")
                do {
                    if data.isEmpty {
                        // Not all requests return JSON data
                        jsonResponse = [:]
                    } else {
                        jsonResponse = try JSONSerialization.jsonObject(with: data) as? [String: Any] ?? [:]
                    }
                } catch {
                    XCTFail("Failed to deserialize JSON response")
                }
            } else {
                XCTFail("Request failed with status code \(response.statusCode)")
            }

            completionHandlerInvokedExpectation.fulfill()
        }

        task.resume()
        let waitResult = XCTWaiter().wait(for: [completionHandlerInvokedExpectation], timeout: 10)
        XCTAssertEqual(waitResult, .completed, "Waiting for partner API request expectation completed")
        XCTAssertNil(requestError)

        return jsonResponse
    }
}
