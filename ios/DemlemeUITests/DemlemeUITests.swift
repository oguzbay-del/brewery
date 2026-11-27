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

    @MainActor
    private func scan(_ code: String, in app: XCUIApplication) {
        let field = app.textFields["scan.code_field"]
        XCTAssertTrue(field.waitForExistence(timeout: 5))
        field.tap()
        field.typeText(code)
        app.buttons["scan.resolve"].tap()
    }

    @MainActor
    func testScanShowsCardWithOnlyPermittedActions() throws {
        let app = launch(modules: "core:on,production:on")
        scan("KEG-1", in: app)
        XCTAssertTrue(app.descendants(matching: .any)["scan.card"].waitForExistence(timeout: 5))
        XCTAssertTrue(app.descendants(matching: .any)["scan.action.change_status"].exists)
        XCTAssertFalse(app.descendants(matching: .any)["scan.action.fill"].exists)
        XCTAssertFalse(app.descendants(matching: .any)["scan.action.retire"].exists)
    }

    @MainActor
    func testScanCardWithoutActionsSaysSo() throws {
        let app = launch(modules: "core:on,production:on")
        scan("LOT-1", in: app)
        XCTAssertTrue(app.staticTexts["scan.no_actions"].waitForExistence(timeout: 5))
    }

    @MainActor
    func testUnknownCodeShowsError() throws {
        let app = launch(modules: "core:on,production:on")
        scan("NOPE", in: app)
        XCTAssertTrue(app.staticTexts["scan.error"].waitForExistence(timeout: 5))
    }

    @MainActor
    func testOffProductionModuleHidesScan() throws {
        let app = launch(modules: "core:on,production:off")
        XCTAssertTrue(app.staticTexts["app.title"].waitForExistence(timeout: 5))
        XCTAssertFalse(app.textFields["scan.code_field"].exists)
    }
}
