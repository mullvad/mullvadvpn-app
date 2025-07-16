//
//  AppLanguage.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-07-16.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
enum AppLanguage: String, CaseIterable, Identifiable {
    case english = "en"
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
    case chineseSimplified = "zh-Hans" // Maps to zh-CN
    case chineseTraditional = "zh-Hant" // Maps to zh-TW

    var id: String { rawValue }

    var displayName: String {
        switch self {
        case .english: return "English"
        case .danish: return "Dansk"
        case .german: return "Deutsch"
        case .spanish: return "Español"
        case .finnish: return "Suomi"
        case .french: return "Français"
        case .italian: return "Italiano"
        case .japanese: return "日本語"
        case .korean: return "한국어"
        case .burmese: return "မြန်မာ"
        case .norwegianBokmal: return "Norsk Bokmål"
        case .dutch: return "Nederlands"
        case .polish: return "Polski"
        case .portuguese: return "Português"
        case .russian: return "Русский"
        case .swedish: return "Svenska"
        case .thai: return "ไทย"
        case .turkish: return "Türkçe"
        case .chineseSimplified: return "简体中文"
        case .chineseTraditional: return "繁體中文"
        }
    }

    var countryCodeForFlag: String {
        switch self {
        case .english: return "us" // English → US flag (or "gb" for UK)
        case .danish: return "dk"
        case .german: return "de"
        case .spanish: return "es"
        case .finnish: return "fi"
        case .french: return "fr"
        case .italian: return "it"
        case .japanese: return "jp"
        case .korean: return "kr"
        case .burmese: return "mm"
        case .norwegianBokmal: return "no"
        case .dutch: return "nl"
        case .polish: return "pl"
        case .portuguese: return "pt"
        case .russian: return "ru"
        case .swedish: return "se"
        case .thai: return "th"
        case .turkish: return "tr"
        case .chineseSimplified: return "cn"
        case .chineseTraditional: return "tw"
        }
    }

    static var allSorted: [AppLanguage] {
        AppLanguage.allCases
            .sorted { $0.displayName.localizedCaseInsensitiveCompare($1.displayName) == .orderedAscending }
    }

    static func from(_ code: String) -> AppLanguage {
        AppLanguage(rawValue: code) ?? .english
    }

    var flagEmoji: String {
        let base: UInt32 = 127397
        var flagString = ""
        for scalar in countryCodeForFlag.uppercased().unicodeScalars {
            guard let scalarValue = UnicodeScalar(base + scalar.value) else { return "" }
            flagString.unicodeScalars.append(scalarValue)
        }
        return flagString
    }

    static var currentLanguage: AppLanguage {
        let defaultCode = AppLanguage.english.rawValue
        let fullCode = Locale.preferredLanguages.first ?? defaultCode

        if #available(iOS 16, *) {
            let locale = Locale(identifier: fullCode)
            if let script = locale.language.script?.identifier {
                switch script {
                case "Hans":
                    return .chineseSimplified
                case "Hant":
                    return .chineseTraditional
                default:
                    break
                }
            }
        } else {
            if fullCode.contains("Hans") {
                return .chineseSimplified
            } else if fullCode.contains("Hant") {
                return .chineseTraditional
            }
        }

        // Otherwise, try to get languageCode (e.g., "en", "fr")
        let locale = Locale(identifier: fullCode)
        let langCode = locale.languageCode ?? defaultCode

        return AppLanguage.from(langCode)
    }
}
