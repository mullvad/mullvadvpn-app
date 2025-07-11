//
//  LanguageListView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-07-11.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct LanguageListView: View {
    @ObservedObject var localizationManager: LocalizationManager

    var body: some View {
        NavigationView {
            MullvadList(AppLanguage.allSorted) { language in
                HStack {
                    HStack {
                        Text(language.flagEmoji)
                            .font(.mullvadSmallSemiBold)
                        Text(language.displayName)
                            .font(.mullvadSmall)
                        Spacer()
                        if language == localizationManager.selectedLanguage {
                            Image(uiImage: UIImage.tick)
                        }
                    }
                }
                .contentShape(Rectangle()) // make whole row tappable
                .onTapGesture {
                    localizationManager.selectedLanguage = language
                }
            }
            .navigationTitle("Select Language")
        }
    }
}

#Preview {
    let manager = LocalizationManager()
    manager.selectedLanguage = .french
    return LanguageListView(localizationManager: manager)
        .environment(\.locale, Locale(identifier: manager.selectedLanguage.rawValue))
}
