//
//  LocalizationManager.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-07-11.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation
import SwiftUI

final class LocalizationManager: ObservableObject {
    private static let languageKey = "SelectedAppLanguage"

    @AppStorage(languageKey) private var storedLanguageCode: String = AppLanguage.english.rawValue

    // Published property for UI to react on changes
    @Published var selectedLanguage: AppLanguage {
        didSet {
            storedLanguageCode = selectedLanguage.rawValue
            applyLanguageOverrideIfNeeded()
        }
    }

    // MARK: - Debug Feature Toggle (replace this with real feature flag system if needed)

    private static var isLanguageOverrideEnabled: Bool {
        #if DEBUG
        return true // Enable in development builds
        #else
        return false // Disabled in production
        #endif
    }

    init() {
        let savedCode = UserDefaults.standard.string(forKey: LocalizationManager.languageKey) ?? AppLanguage.english
            .rawValue
        selectedLanguage = AppLanguage(rawValue: savedCode) ?? .english
        applyLanguageOverrideIfNeeded()
    }

    private func applyLanguageOverrideIfNeeded() {
        guard LocalizationManager.isLanguageOverrideEnabled else {
            UserDefaults.standard.removeObject(forKey: "AppleLanguages")
            UserDefaults.standard.synchronize()
            return
        }
        UserDefaults.standard.set([selectedLanguage.rawValue], forKey: "AppleLanguages")
        UserDefaults.standard.synchronize()
    }
}
