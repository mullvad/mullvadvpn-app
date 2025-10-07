import SwiftUI

extension Color {
    private static let mullvadPrimaryColor = MullvadBlue.base
    private static let mullvadSecondaryColor = MullvadDarkBlue.base
    private static let mullvadWarningColor = UIColor.warningColor.color
    private static let mullvadDangerColor = UIColor.dangerColor.color
    static let mullvadSuccessColor = UIColor.successColor.color

    static let mullvadBackground: Color = .mullvadSecondaryColor
    static let mullvadTextPrimary: Color = UIColor.primaryTextColor.color
    static let mullvadTextSecondary: Color = MullvadWhite._60
    static let mullvadTextPrimaryDisabled: Color = .mullvadTextPrimary.opacity(
        0.2
    )
    static let secondaryTextColor: Color = UIColor.secondaryTextColor.color

    private enum MullvadBlue {
        static let base: Color = .init(red: 0.16, green: 0.3, blue: 0.45)
        static let _10: Color = .init(red: 0.11, green: 0.19, blue: 0.29)
        static let _20: Color = .init(red: 0.11, green: 0.2, blue: 0.31)
        static let _40: Color = .init(red: 0.12, green: 0.23, blue: 0.34)
        static let _50: Color = .init(red: 0.11, green: 0.2, blue: 0.31)
        static let _60: Color = .init(red: 0.14, green: 0.25, blue: 0.38)
        static let _80: Color = .init(red: 0.15, green: 0.28, blue: 0.42)
    }

    private enum MullvadDarkBlue {
        static let base: Color = .init(red: 0.10, green: 0.18, blue: 0.27)
    }

    private enum MullvadRed {
        static let base: Color = .init(red: 0.89, green: 0.25, blue: 0.22)
    }

    private enum MullvadGreen {
        static let base: Color = .init(red: 0.27, green: 0.68, blue: 0.3)
    }

    private enum MullvadYellow {
        static let base: Color = .init(red: 1, green: 0.84, blue: 0.14)
    }

    private enum MullvadWhiteOnDarkBlue {
        static let _5: Color = .init(red: 0.15, green: 0.22, blue: 0.31)
    }

    private enum MullvadWhite {
        static let _100: Color = .white
        static let _80: Color = _100.opacity(0.8)
        static let _60: Color = _100.opacity(0.6)
        static let _40: Color = _100.opacity(0.4)
        static let _20: Color = _100.opacity(0.2)
    }

    enum MullvadText {
        static let inputPlaceholder: Color = MullvadWhite._60
        static let disabled: Color = MullvadWhite._20
        static let onBackground: Color = MullvadWhite._60
        static let onBackgroundEmphasis100: Color = MullvadWhite._100
    }

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
        static let background: Color = .mullvadPrimaryColor
        enum Item {
            static let parent: Color = .mullvadPrimaryColor
            static let child1 = Color.MullvadBlue._60
            static let child2 = Color.MullvadBlue._40
            static let child3 = Color.MullvadBlue._20
            static let child4 = Color.MullvadBlue._10
        }
    }

    enum MullvadTextField {
        static let background: Color = .MullvadBlue._40
        static let backgroundDisabled: Color = .MullvadWhiteOnDarkBlue._5
        static let backgroundSuggestion: Color = .MullvadBlue._80
        static let inputPlaceholder: Color = MullvadText.inputPlaceholder
        static let textDisabled: Color = MullvadText.disabled
        static let textInput: Color = MullvadText.onBackgroundEmphasis100
        static let label: Color = MullvadText.onBackgroundEmphasis100
        static let border: Color = .MullvadOpacities.chalk40
        static let borderFocused: Color = .MullvadNewGraphicalProfile.chalk
        static let borderError: Color = .MullvadNewGraphicalProfile.red
    }

    private enum MullvadOpacities {
        static let chalk40: Color = .MullvadNewGraphicalProfile.chalk.opacity(
            0.4
        )
    }

    private enum MullvadNewGraphicalProfile {
        static let red: Color = .init(red: 0.92, green: 0.36, blue: 0.25)
        static let chalk: Color = .init(red: 0.97, green: 0.97, blue: 0.95)
        static let dark: Color = .init(red: 0.31, green: 0.29, blue: 0.29)
    }
}
