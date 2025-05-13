import SwiftUI

extension Color {
    private static let mullvadPrimaryColor = UIColor.primaryColor.color
    private static let mullvadSecondaryColor = UIColor.secondaryColor.color
    private static let mullvadWarningColor = UIColor.warningColor.color
    private static let mullvadDangerColor = UIColor.dangerColor.color
    private static let mullvadSuccessColor = UIColor.successColor.color

    static let mullvadBackground: Color = .mullvadSecondaryColor
    static let mullvadTextPrimary: Color = UIColor.primaryTextColor.color
    static let mullvadTextPrimaryDisabled: Color = .mullvadPrimaryColor.opacity(
        0.2
    )

    enum MullvadButton {
        static let primary: Color = .mullvadPrimaryColor
        static let primaryPressed = Color(red: 0.12, green: 0.23, blue: 0.34)
        static let primaryDisabled = primaryPressed
        static let danger: Color = .mullvadDangerColor
        static let dangerPressed = Color(red: 0.42, green: 0.21, blue: 0.25)
        static let dangerDisabled = dangerPressed
        static let positive: Color = .mullvadSuccessColor
        static let positivePressed = Color(red: 0.16, green: 0.38, blue: 0.28)
        static let positiveDisabled = positivePressed
    }

    enum MullvadList {
        static let separator: Color = .mullvadSecondaryColor
    }
}
