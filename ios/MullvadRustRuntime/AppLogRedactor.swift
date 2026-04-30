//
//  AppLogRedactor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging

public final class AppLogRedactor: AppLogRedactorProtocol {
    let containerPaths: [URL]
    let customStrings: [String]

    public required init(_ containerPaths: [URL] = [], customStrings: [String] = []) {
        self.containerPaths = containerPaths
        self.customStrings = customStrings
    }

    public func redact(_ input: String) -> String {
        let logRedactor = RustLogRedactor()
        let rules: [RedactionRules] = [
            .accountNumbers,
            .ipv4,
            .ipv6,
            .customStrings(customStrings),
            .containerPaths(containerPaths),
        ]
        return logRedactor.redact(input, using: rules)
    }
}
