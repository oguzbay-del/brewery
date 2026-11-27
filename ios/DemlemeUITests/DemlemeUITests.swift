import XCTest

final class DemlemeUITests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    /// Launches the app in English with a fixed module state ("id:on|off|ro,...").
    @MainActor
    private func launch(modules: String) -> XCUIApplication {
        let app = XCUIApplication()
        app.launchArguments += ["-AppleLanguages", "(en)", "-AppleLocale", "en_US"]
        app.launchEnvironment["GRISTORA_UITEST_MODULES"] = modules
        app.launch()
        return app
    }

    @MainActor
    func testLaunchShowsTitle() throws {
        let app = launch(modules: "core:on,production:on")
        let title = app.staticTexts["app.title"]
        XCTAssertTrue(title.waitForExistence(timeout: 5))
        XCTAssertEqual(title.label, "GRISTORA")
    }

    @MainActor
    func testOffModuleHidesItsTab() throws {
        let app = launch(modules: "core:on,production:off")
        XCTAssertTrue(app.tabBars.buttons["Recipes"].waitForExistence(timeout: 5))
        XCTAssertFalse(app.tabBars.buttons["Kegs"].exists)
    }

    @MainActor
    func testOffModuleHidesRecipesAndBatches() throws {
        let app = launch(modules: "core:off,production:on")
        XCTAssertTrue(app.tabBars.buttons["Kegs"].waitForExistence(timeout: 5))
        XCTAssertFalse(app.tabBars.buttons["Recipes"].exists)
        XCTAssertFalse(app.tabBars.buttons["Batches"].exists)
    }

    @MainActor
    func testReadOnlyModuleShowsBannerAndDisablesWrite() throws {
        let app = launch(modules: "core:on,production:ro")
        let tab = app.tabBars.buttons["Kegs"]
        XCTAssertTrue(tab.waitForExistence(timeout: 5))
        tab.tap()
        XCTAssertTrue(app.staticTexts["module.read_only_banner"].waitForExistence(timeout: 5))
        let write = app.buttons["module.write_button"]
        XCTAssertTrue(write.exists)
        XCTAssertFalse(write.isEnabled)
    }

    @MainActor
    func testSettingsListsModules() throws {
        let app = launch(modules: "core:on,production:off")
        let tab = app.tabBars.buttons["Settings"]
        XCTAssertTrue(tab.waitForExistence(timeout: 5))
        tab.tap()
        let row = app.descendants(matching: .any)["module.row.production"]
        XCTAssertTrue(row.waitForExistence(timeout: 5))
    }
}
