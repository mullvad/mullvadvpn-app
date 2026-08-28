// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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

    enum AllowedInput: Equatable {
        case numeric
        case alphanumeric(isUpperCase: Bool)
        case passphrase
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
        let effectiveGroupSize = (configuration.allowedInput == .passphrase && (text.first?.isLetter ?? false)) ? 0 : configuration.groupSize
        return effectiveGroupSize > 0 ? splitIntoGroups(normalizedText) : normalizedText
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
        case .passphrase:
            input.filter { character in
                character.isLetter || character.isNumber || character == " "
            }
        }
    }

    func normalize(_ input: String) -> String {
        switch configuration.allowedInput {
        case .numeric:
            input
        case .alphanumeric(let isUpperCase):
            isUpperCase ? input.uppercased() : input.lowercased()
        case .passphrase:
            input.lowercased()
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

extension GroupedTextFormatter {
    static let accountNumber = GroupedTextFormatter(
        configuration: .init(
            allowedInput: .passphrase,
            groupSeparator: " ",
            groupSize: 4,
            maxGroups: 4
        )
    )
}
