import SwiftUI

enum MultihopContext: CaseIterable, CustomStringConvertible, Hashable {
    case entry, exit

    var description: String {
        switch self {
        case .entry:
            "Entry"
        case .exit:
            "Exit"
        }
    }
}
