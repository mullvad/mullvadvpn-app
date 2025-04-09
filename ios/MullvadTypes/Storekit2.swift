public struct StorekitTransaction: Codable {
    let transaction: String

    public init(transaction: String) {
        self.transaction = transaction
    }
}
