import SwiftUI

extension Font {
    private struct Size {
        static let big: CGFloat = 34
        static let large: CGFloat = 28
        static let medium: CGFloat = 20
        static let small: CGFloat = 17
        static let tiny: CGFloat = 15
        static let mini: CGFloat = 13
    }

    static let mullvadBig: Font = .system(
        size: Size.big,
        weight: .bold,
        design: .default
    )
    static let mullvadLarge: Font = .system(
        size: Size.large,
        weight: .bold, design: .default
    )
    static let mullvadMedium: Font = .system(
        size: Size.medium,
        weight: .semibold,
        design: .default
    )
    static let mullvadSmallSemiBold: Font = .system(
        size: Size.small,
        weight: .semibold,
        design: .default
    )
    static let mullvadSmall: Font = .system(
        size: Size.small,
        weight: .regular,
        design: .default
    )
    static let mullvadTiny: Font = .system(
        size: Size.tiny,
        weight: .regular,
        design: .default
    )
    static let mullvadTinySemiBold: Font = .system(
        size: Size.tiny,
        weight: .semibold,
        design: .default
    )
    static let mullvadMiniSemiBold: Font = .system(
        size: Size.mini,
        weight: .semibold,
        design: .default
    )
    static let mullvadMini: Font = .system(
        size: Size.mini,
        weight: .regular,
        design: .default
    )
}
