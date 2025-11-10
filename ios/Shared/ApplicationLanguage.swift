//
//  ApplicationLanguage.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-07-16.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

/**
 * TODO:
 * Edit the "Localization Cleanup (Release Build)" build script phase after
 * multi-language support is completed and released.
 *
 * Note:
 * - Localization is not available for the Staging configuration, which is used by `UITest`.
 * - When the functionality is finished, the script should:
 *    • Remove bilingual content only for Staging.
 *    • Eliminate the Debug configuration check.
 */
enum ApplicationLanguage: String, CaseIterable, Identifiable {
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
    case chineseSimplified = "zh-Hans"  // Maps to zh-CN
    case chineseTraditional = "zh-Hant"  // Maps to zh-TW

    var id: String { rawValue }

    var displayName: String {
        switch self {
        case .english: "English"
        case .danish: "Dansk"
        case .german: "Deutsch"
        case .spanish: "Español"
        case .finnish: "Suomi"
        case .french: "Français"
        case .italian: "Italiano"
        case .japanese: "日本語"
        case .korean: "한국어"
        case .burmese: "မြန်မာ"
        case .norwegianBokmal: "Norsk Bokmål"
        case .dutch: "Nederlands"
        case .polish: "Polski"
        case .portuguese: "Português"
        case .russian: "Русский"
        case .swedish: "Svenska"
        case .thai: "ไทย"
        case .turkish: "Türkçe"
        case .chineseSimplified: "简体中文"
        case .chineseTraditional: "繁體中文"
        }
    }

    var countryCodeForFlag: String {
        switch self {
        case .english: "us"  // English → US flag (or "gb" for UK)
        case .danish: "dk"
        case .german: "de"
        case .spanish: "es"
        case .finnish: "fi"
        case .french: "fr"
        case .italian: "it"
        case .japanese: "jp"
        case .korean: "kr"
        case .burmese: "mm"
        case .norwegianBokmal: "no"
        case .dutch: "nl"
        case .polish: "pl"
        case .portuguese: "pt"
        case .russian: "ru"
        case .swedish: "se"
        case .thai: "th"
        case .turkish: "tr"
        case .chineseSimplified: "cn"
        case .chineseTraditional: "tw"
        }
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

    static var currentLanguage: ApplicationLanguage {
        let defaultCode = ApplicationLanguage.english.rawValue
        let fullCode = Locale.preferredLanguages.first ?? defaultCode

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
        let langCode = locale.language.languageCode?.identifier ?? defaultCode

        return ApplicationLanguage(rawValue: langCode) ?? .english
    }
}
