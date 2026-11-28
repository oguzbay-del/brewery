import XCTest

/// iPad visual checks (UIQ-11): screenshots as attachments and accessibility audits.
/// Run on iPad Pro 11-inch and 13-inch. The tests skip on an iPhone.
final class IPadVisualUITests: XCTestCase {
    private static let orientations: [(String, UIDeviceOrientation)] = [
        ("landscape", .landscapeLeft),
        ("portrait", .portrait),
    ]

    override func setUpWithError() throws {
        continueAfterFailure = true
        try XCTSkipUnless(UIDevice.current.userInterfaceIdiom == .pad, "iPad only")
    }

    @MainActor
    override func tearDown() async throws {
        XCUIDevice.shared.orientation = .portrait
    }

    @MainActor
    private func launch(orientation: UIDeviceOrientation, gallery: Bool) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchArguments += ["-AppleLanguages", "(en)", "-AppleLocale", "en_US"]
        if gallery { app.launchArguments.append("-KitGallery") }
        app.launchEnvironment["GRISTORA_UITEST_MODULES"] = "core:on,production:on"
        XCUIDevice.shared.orientation = orientation
        app.launch()
        return app
    }

    @MainActor
    private func attach(_ name: String, from app: XCUIApplication) {
        let attachment = XCTAttachment(screenshot: app.screenshot())
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }

    /// The window must contain every hittable element.
    @MainActor
    private func assertFramesInsideWindow(_ app: XCUIApplication, file: StaticString = #filePath, line: UInt = #line) {
        let window = app.windows.firstMatch.frame
        for element in app.descendants(matching: .any).allElementsBoundByIndex where element.isHittable {
            let frame = element.frame
            XCTAssertTrue(
                window.insetBy(dx: -1, dy: -1).contains(frame),
                "\(element.identifier) \(element.label) is outside the window: \(frame)",
                file: file,
                line: line
            )
        }
    }

    @MainActor
    func testKitGallery() throws {
        for (name, orientation) in Self.orientations {
            let app = launch(orientation: orientation, gallery: true)
            XCTAssertTrue(app.staticTexts["Buttons"].waitForExistence(timeout: 5))
            attach("kit-gallery-\(name)", from: app)
            // Documented exclusion: the gallery has long text on purpose. It wraps and then truncates.
            try app.performAccessibilityAudit(for: .all.subtracting(.textClipped))
            app.terminate()
        }
    }

    @MainActor
    func testMainScreens() throws {
        for (name, orientation) in Self.orientations {
            let app = launch(orientation: orientation, gallery: false)
            XCTAssertTrue(app.staticTexts["app.title"].waitForExistence(timeout: 5))
            attach("home-\(name)", from: app)
            try app.performAccessibilityAudit()
            assertFramesInsideWindow(app)

            let settings = app.tabBars.buttons["Settings"]
            XCTAssertTrue(settings.waitForExistence(timeout: 5))
            settings.tap()
            XCTAssertTrue(app.descendants(matching: .any)["module.row.production"].waitForExistence(timeout: 5))
            attach("settings-\(name)", from: app)
            try app.performAccessibilityAudit()
            assertFramesInsideWindow(app)
            app.terminate()
        }
    }
}
