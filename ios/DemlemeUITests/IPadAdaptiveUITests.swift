import XCTest

/// iPad only. Run on an iPad simulator; the tests skip on an iPhone.
final class IPadAdaptiveUITests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
        try XCTSkipUnless(UIDevice.current.userInterfaceIdiom == .pad, "iPad only")
    }

    @MainActor
    override func tearDown() async throws {
        XCUIDevice.shared.orientation = .portrait
    }

    @MainActor
    func testSelectionAndTabSurviveRotation() throws {
        let app = XCUIApplication()
        app.launchArguments += ["-AppleLanguages", "(en)", "-AppleLocale", "en_US"]
        app.launchEnvironment["GRISTORA_UITEST_MODULES"] = "core:on,production:on"
        XCUIDevice.shared.orientation = .landscapeLeft
        app.launch()

        let tab = app.tabBars.buttons["Settings"]
        XCTAssertTrue(tab.waitForExistence(timeout: 5))
        tab.tap()
        let row = app.descendants(matching: .any)["module.row.production"]
        XCTAssertTrue(row.waitForExistence(timeout: 5))
        row.tap()
        XCTAssertTrue(app.descendants(matching: .any)["module.detail"].waitForExistence(timeout: 5))

        for orientation in [UIDeviceOrientation.portrait, .landscapeRight, .portrait] {
            XCUIDevice.shared.orientation = orientation
            XCTAssertTrue(
                app.descendants(matching: .any)["module.detail"].waitForExistence(timeout: 5),
                "The selection is lost after rotation to \(orientation.rawValue)"
            )
            XCTAssertFalse(app.descendants(matching: .any)["module.detail.empty"].exists)
        }
    }
}
