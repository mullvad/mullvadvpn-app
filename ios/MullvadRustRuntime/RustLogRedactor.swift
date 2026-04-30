//
//  RustAppLogConsolidation.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import MullvadLogging

public class RustLogRedactor: LogRedactorProtocol {
    private let logRedactor: LogRedactor

    deinit {
        drop_log_redactor(logRedactor)
    }

    public init() {
        self.logRedactor = init_log_redactor()
    }

    public func redact(_ input: String, using rules: [RedactionRules]) -> String {
        let swiftConfig = makeConfig(from: rules)
        return swiftConfig.withRustConfig { redactionConfig in
            let resultPtr = redact_log(logRedactor, input, &redactionConfig)
            guard let resultPtr else {
                return input
            }

            defer {
                free_rust_string(resultPtr)
            }

            let result = String(cString: resultPtr)
            return result
        }
    }

    private func makeConfig(from rules: [RedactionRules]) -> SwiftRedactionConfig {
        var config = SwiftRedactionConfig()

        for rule in rules {
            switch rule {
            case .accountNumbers:
                config.redactAccountNumbers = true

            case .ipv4:
                config.redactIPv4 = true

            case .ipv6:
                config.redactIPv6 = true

            case .containerPaths(let urls):
                config.containerPaths.append(contentsOf: urls.map { $0.path })

            case .customStrings(let strings):
                config.customStrings.append(contentsOf: strings)
            }
        }

        return config
    }
}

private struct SwiftRedactionConfig {
    var redactAccountNumbers = false
    var redactIPv4 = false
    var redactIPv6 = false
    var containerPaths: [String] = []
    var customStrings: [String] = []

    func withRustConfig<T>(
        _ body: (inout RedactionConfig) -> T
    ) -> T {
        return containerPaths.withCStringArray { containerPtrs, containerLen in
            return customStrings.withCStringArray { customPtrs, customLen in
                var config = RedactionConfig(
                    redact_account_numbers: redactAccountNumbers,
                    redact_ipv4: redactIPv4,
                    redact_ipv6: redactIPv6,
                    container_paths: containerPtrs,
                    container_paths_len: containerLen,
                    custom_strings: customPtrs,
                    custom_strings_len: customLen
                )

                return body(&config)
            }
        }
    }
}
