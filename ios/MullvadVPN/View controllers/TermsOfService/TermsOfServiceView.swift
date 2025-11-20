//
//  TermsOfServiceView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2025-06-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct TermsOfServiceView: View {
    public var agreeToTermsAndServices: (() -> Void)?
    let padding = EdgeInsets(top: 24, leading: 16, bottom: 24, trailing: 16)
    @ScaledMetric(relativeTo: .footnote)
    var imageHeight = 20

    let termsOfService = LocalizedStringKey(
        """
        You have a right to privacy. That’s why we never store activity logs, don’t ask for personal \
        information, and encourage anonymous payments.

        In some situations, as outlined in our privacy policy, we might process personal data that you \
        choose to send, for example if you email us.

        We strongly believe in retaining as little data as possible because we want you to remain anonymous.
        """)

    let privacyPolicyText = NSLocalizedString("privacy policy", comment: "").localizedCapitalized
    var scrollableContent: some View {
        ScrollView {
            Text(LocalizedStringKey("Do you agree to remaining anonymous?"))
                .font(.mullvadLarge)
                .foregroundStyle(.white)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.bottom, 16)
            Text(termsOfService)
                .font(.mullvadSmall)
                .foregroundStyle(Color.secondaryTextColor)
        }
        .padding(padding)
    }

    var body: some View {
        VStack(alignment: .leading) {
            scrollableContent.scrollBounceBehavior(.basedOnSize)
            HStack {
                Text(
                    LocalizedStringKey(
                        stringLiteral: "[\(privacyPolicyText)](\(ApplicationConfiguration.privacyPolicyLink))")
                )
                .font(.mullvadSmall)
                .underline(true, color: .white)
                .foregroundStyle(.white)
                .tint(.white)
                Image(uiImage: UIImage.iconExtLink)
                    .resizable()
                    .scaledToFit()
                    .frame(height: imageHeight)
                    .foregroundStyle(.white)
            }
            .padding(padding)
            MainButton(
                text: LocalizedStringKey("Agree and continue"),
                style: .default,
                action: agreeToTermsAndServices ?? {}
            )
            .accessibilityIdentifier(AccessibilityIdentifier.agreeButton.asString)
            .padding(padding)
            .background(Color(UIColor.secondaryColor))
        }
        .background(Color(UIColor.primaryColor))
    }
}

#Preview {
    TermsOfServiceView()
        .frame(maxWidth: .infinity, maxHeight: .infinity)
}
