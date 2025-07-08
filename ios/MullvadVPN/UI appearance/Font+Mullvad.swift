import SwiftUI

extension Font {
    static var mullvadBig: Font { .largeTitle.bold() }
    static var mullvadLarge: Font { .title.bold() }
    static var mullvadMedium: Font { .title3.weight(.semibold) }
    static var mullvadSmall: Font { .body }
    static var mullvadSmallSemiBold: Font { mullvadSmall.weight(.semibold) }
    static var mullvadTiny: Font { .subheadline }
    static var mullvadTinySemiBold: Font { .mullvadTiny.weight(.semibold) }
    static var mullvadMini: Font { .footnote }
    static var mullvadMiniSemiBold: Font { mullvadMini.weight(.semibold) }
}

extension UIFont {
    static var mullvadBig: UIFont { .preferredFont(forTextStyle: .largeTitle, weight: .bold) }
    static var mullvadLarge: UIFont { .preferredFont(forTextStyle: .title1, weight: .bold) }
    static var mullvadMedium: UIFont { .preferredFont(forTextStyle: .title3, weight: .semibold) }
    static var mullvadSmall: UIFont { .preferredFont(forTextStyle: .body) }
    static var mullvadSmallSemiBold: UIFont { .preferredFont(forTextStyle: .body, weight: .semibold) }
    static var mullvadTiny: UIFont { .preferredFont(forTextStyle: .subheadline) }
    static var mullvadTinySemiBold: UIFont { .preferredFont(forTextStyle: .subheadline, weight: .semibold) }
    static var mullvadMini: UIFont { .preferredFont(forTextStyle: .footnote) }
    static var mullvadMiniSemiBold: UIFont { .preferredFont(forTextStyle: .footnote, weight: .semibold) }
}
