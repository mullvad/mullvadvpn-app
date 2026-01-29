//
//  LogRedactor.swift
//  MullvadLogging
//
//  Created by Emīls on 2026-01-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

private let kRedactedPlaceholder = "[REDACTED]"
private let kRedactedAccountPlaceholder = "[REDACTED ACCOUNT NUMBER]"

/// A thread-safe utility for redacting sensitive information from log messages.
///
/// This class provides on-the-fly redaction of:
/// - IPv4 addresses
/// - IPv6 addresses
/// - Account numbers (16-digit sequences)
///
/// The regex patterns are compiled once at initialization and reused for all redaction operations.
/// `NSRegularExpression` is thread-safe for matching operations after compilation.
public final class LogRedactor: Sendable {
    /// Shared singleton instance with pre-compiled regex patterns.
    public static let shared = LogRedactor()

    private let ipv4Regex: NSRegularExpression
    private let ipv6Regex: NSRegularExpression
    private let accountNumberRegex: NSRegularExpression

    private init() {
        // IPv4 pattern from https://www.regular-expressions.info/ip.html
        let ipv4Pattern = #"""
            \b(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.
              (25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.
              (25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.
              (25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\b
            """#

        // IPv6 pattern from https://stackoverflow.com/a/17871737
        let ipv6Pattern = #"""
            # IPv6 RegEx
            (
            ([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|          # 1:2:3:4:5:6:7:8
            ([0-9a-fA-F]{1,4}:){1,7}:|                         # 1::                              1:2:3:4:5:6:7::
            ([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|         # 1::8             1:2:3:4:5:6::8  1:2:3:4:5:6::8
            ([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|  # 1::7:8           1:2:3:4:5::7:8  1:2:3:4:5::8
            ([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|  # 1::6:7:8         1:2:3:4::6:7:8  1:2:3:4::8
            ([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|  # 1::5:6:7:8       1:2:3::5:6:7:8  1:2:3::8
            ([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|  # 1::4:5:6:7:8     1:2::4:5:6:7:8  1:2::8
            [0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|       # 1::3:4:5:6:7:8   1::3:4:5:6:7:8  1::8
            :((:[0-9a-fA-F]{1,4}){1,7}|:)|                     # ::2:3:4:5:6:7:8  ::2:3:4:5:6:7:8 ::8       ::
            fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|     # fe80::7:8%eth0   fe80::7:8%1     (link-local IPv6 addresses with zone index)
            ::(ffff(:0{1,4}){0,1}:){0,1}
            ((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}
            (25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|          # ::255.255.255.255   ::ffff:255.255.255.255  ::ffff:0:255.255.255.255  (IPv4-mapped IPv6 addresses and IPv4-translated addresses)
            ([0-9a-fA-F]{1,4}:){1,4}:
            ((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}
            (25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])           # 2001:db8:3:4::192.0.2.33  64:ff9b::192.0.2.33 (IPv4-Embedded IPv6 Address)
            )
            """#

        // Account number pattern: 16 consecutive digits
        let accountNumberPattern = #"\d{16}"#

        // swift-format-ignore: NeverUseForceTry
        ipv4Regex = try! NSRegularExpression(pattern: ipv4Pattern, options: [.allowCommentsAndWhitespace])
        // swift-format-ignore: NeverUseForceTry
        ipv6Regex = try! NSRegularExpression(pattern: ipv6Pattern, options: [.allowCommentsAndWhitespace])
        // swift-format-ignore: NeverUseForceTry
        accountNumberRegex = try! NSRegularExpression(pattern: accountNumberPattern)
    }

    /// Redacts all sensitive information from the given string.
    ///
    /// This method applies all redaction patterns in order:
    /// 1. IPv4 addresses
    /// 2. IPv6 addresses
    /// 3. Account numbers (16-digit sequences)
    ///
    /// - Parameter string: The string to redact.
    /// - Returns: A new string with sensitive information replaced by placeholders.
    public func redact(_ string: String) -> String {
        var result = string
        result = redactIPv4(result)
        result = redactIPv6(result)
        result = redactAccountNumber(result)
        return result
    }

    /// Redacts only IPv4 addresses from the given string.
    ///
    /// - Parameter string: The string to redact.
    /// - Returns: A new string with IPv4 addresses replaced by `[REDACTED]`.
    public func redactIPv4(_ string: String) -> String {
        replaceMatches(of: ipv4Regex, in: string, with: kRedactedPlaceholder)
    }

    /// Redacts only IPv6 addresses from the given string.
    ///
    /// This is exposed publicly for benchmarking purposes, as IPv6 regex is known to be slower.
    ///
    /// - Parameter string: The string to redact.
    /// - Returns: A new string with IPv6 addresses replaced by `[REDACTED]`.
    public func redactIPv6(_ string: String) -> String {
        replaceMatches(of: ipv6Regex, in: string, with: kRedactedPlaceholder)
    }

    /// Redacts only account numbers (16-digit sequences) from the given string.
    ///
    /// - Parameter string: The string to redact.
    /// - Returns: A new string with account numbers replaced by `[REDACTED ACCOUNT NUMBER]`.
    public func redactAccountNumber(_ string: String) -> String {
        replaceMatches(of: accountNumberRegex, in: string, with: kRedactedAccountPlaceholder)
    }

    private func replaceMatches(
        of regex: NSRegularExpression,
        in string: String,
        with replacement: String
    ) -> String {
        let range = NSRange(string.startIndex..<string.endIndex, in: string)
        let template = NSRegularExpression.escapedTemplate(for: replacement)
        return regex.stringByReplacingMatches(in: string, options: [], range: range, withTemplate: template)
    }
}
