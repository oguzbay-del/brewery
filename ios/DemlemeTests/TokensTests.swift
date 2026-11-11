@testable import Demleme
import Testing
import UIKit

/// Tests are hosted in the app, so Bundle.main is the app bundle.
struct TokensTests {
    @Test func tokenListIsNotEmpty() {
        #expect(!DemlemeColor.allNames.isEmpty)
    }

    @Test func everyTokenResolvesInAssetCatalog() {
        for name in DemlemeColor.allNames {
            #expect(UIColor(named: name, in: Bundle.main, compatibleWith: nil) != nil,
                    "Missing color asset: \(name)")
        }
    }

    @Test func coreTokensExist() {
        let required = ["bg", "card", "label", "tint", "tint-fill", "tint-ink"]
        for name in required {
            #expect(DemlemeColor.allNames.contains(name), "Missing token: \(name)")
        }
    }
}
