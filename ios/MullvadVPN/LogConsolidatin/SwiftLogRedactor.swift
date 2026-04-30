//
//  SwiftLogRedactor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import Foundation
import MullvadLogging

struct SwiftLogRedactor: LogRedactorProtocol {
    private let kRedactedPlaceholder = "[REDACTED]"

    func redact(_ input: String, using rules: [RedactionRules]) -> String {
        return rules.reduce(input) { current, rule in
            switch rule {

            case .accountNumbers:
                return redact(
                    regularExpression: try! NSRegularExpression(pattern: #"\d{16}"#),
                    string: current,
                    replacementString: "[REDACTED ACCOUNT NUMBER]"
                )

            case .containerPaths(let containerPaths):
                return containerPaths.reduce(current) { result, containerURL in
                    result.replacingOccurrences(
                        of: containerURL.path,
                        with: "[REDACTED CONTAINER PATH]"
                    )
                }

            case .ipv4:
                return redact(
                    regularExpression: NSRegularExpression.ipv4RegularExpression,
                    string: current,
                    replacementString: "[REDACTED]"
                )

            case .ipv6:
                return redact(
                    regularExpression: NSRegularExpression.ipv6RegularExpression,
                    string: current,
                    replacementString: "[REDACTED]"
                )

            case .customStrings(let customStrings):
                return customStrings.reduce(current) { result, redact in
                    result.replacingOccurrences(
                        of: redact,
                        with: kRedactedPlaceholder
                    )
                }
            }
        }
    }

    private func redact(
        regularExpression: NSRegularExpression,
        string: String,
        replacementString: String
    ) -> String {
        let nsRange = NSRange(string.startIndex..<string.endIndex, in: string)
        let template = NSRegularExpression.escapedTemplate(for: replacementString)

        return regularExpression.stringByReplacingMatches(
            in: string,
            options: [],
            range: nsRange,
            withTemplate: template
        )
    }
}
