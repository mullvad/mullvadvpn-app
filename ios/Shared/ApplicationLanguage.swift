//
//  ApplicationLanguage.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-07-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum ApplicationLanguage: String, CaseIterable, Identifiable {
    case englishUS = "en-US"
    case englishUK = "en-GB"
    case danish = "da"
    case german = "de"
    case spanish = "es"
    case finnish = "fi"
    case french = "fr"
    case italian = "it"
    case japanese = "ja"
    case korean = "ko"
    case burmese = "my"
    case norwegianBokmal = "nb"
    case dutch = "nl"
    case polish = "pl"
    case portuguese = "pt"
    case russian = "ru"
    case swedish = "sv"
    case thai = "th"
    case turkish = "tr"
    case chineseSimplified = "zh-Hans"
    case chineseTraditional = "zh-Hant"
    case ukrainian = "uk"

    var id: String { rawValue }

    var displayName: String {
        let locale = Locale(identifier: id)
        let name = locale.localizedString(forIdentifier: self.rawValue) ?? "\(self)"
        return name.localizedCapitalized
    }

    static var currentLanguage: ApplicationLanguage {
        let defaultLang: ApplicationLanguage = .englishUS
        let preferred = Locale.preferredLanguages.first ?? defaultLang.rawValue
        let locale = Locale(identifier: preferred)

        let languageCode = locale.language.languageCode?.identifier
        let regionCode = locale.region?.identifier
        let scriptCode = locale.language.script?.identifier

        // Chinese (script + region)
        if languageCode == "zh" {
            if let script = scriptCode {
                switch script {
                case "Hans": return .chineseSimplified
                case "Hant": return .chineseTraditional
                default: break
                }
            }

            if let region = regionCode {
                switch region {
                case "CN", "SG":
                    return .chineseSimplified
                case "TW", "HK", "MO":
                    return .chineseTraditional
                default:
                    break
                }
            }

            return .chineseSimplified
        }

        if languageCode == "en" {
            switch regionCode {
            case "GB": return .englishUK
            case "US": return .englishUS
            default: return .englishUS
            }
        }

        if let lang = languageCode,
            let match = ApplicationLanguage(rawValue: lang)
        {
            return match
        }

        return defaultLang
    }
}
