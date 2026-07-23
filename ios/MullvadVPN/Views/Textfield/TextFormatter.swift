//
//  TextFormatter.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-20.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

protocol TextFormatting {
    func format(_ text: String) -> String
}

extension GroupedTextFormatter {
    struct FormatterConfiguration {
        var allowedInput: AllowedInput
        var groupSeparator: Character
        var groupSize: UInt8
        var maxGroups: UInt8
    }

    enum AllowedInput {
        case numeric
        case alphanumeric(isUpperCase: Bool)
    }
}

struct GroupedTextFormatter: TextFormatting {
    let configuration: FormatterConfiguration

    private var maximumCharacters: Int {
        Int(configuration.groupSize * configuration.maxGroups)
    }

    func format(_ text: String) -> String {
        let filteredText = filterAllowedCharacters(text)
        let normalizedText = normalize(filteredText)
        return splitIntoGroups(normalizedText)
    }

    private func filterAllowedCharacters(_ input: String) -> String {
        return switch configuration.allowedInput {
        case .numeric:
            input.filter { character in
                character.isNumber
            }
        case .alphanumeric:
            input.filter { character in
                character.isLetter || character.isNumber
            }
        }
    }

    func normalize(_ input: String) -> String {
        switch configuration.allowedInput {
        case .numeric:
            input
        case .alphanumeric(let isUpperCase):
            isUpperCase ? input.uppercased() : input.lowercased()
        }
    }

    private func splitIntoGroups(_ input: String) -> String {
        guard !input.isEmpty else { return "" }
        let size = Int(configuration.groupSize)
        let maximumCharacters = Int(configuration.groupSize * configuration.maxGroups)
        let text = String(input.prefix(maximumCharacters))

        return stride(from: 0, to: text.count, by: size)
            .map { start in
                let from = text.index(text.startIndex, offsetBy: start)
                let to =
                    text.index(from, offsetBy: Swift.min(size, text.count - start), limitedBy: text.endIndex)
                    ?? text.endIndex
                return String(text[from..<to])

            }
            .joined(separator: "\(configuration.groupSeparator)")
    }
}
