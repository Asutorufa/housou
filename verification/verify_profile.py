from playwright.sync_api import sync_playwright
import time

def test_profile_modal():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        try:
            print("Navigating to http://localhost:5173")
            page.goto("http://localhost:5173")

            # Wait for page load
            page.wait_for_load_state("networkidle")

            # Login
            print("Finding Login trigger button")
            # Get all buttons in header
            buttons = page.locator("header button").all()
            login_trigger = None
            for btn in buttons:
                label = btn.get_attribute("aria-label")
                if label == "検索を開く":
                    continue
                login_trigger = btn

            if not login_trigger:
                print("Could not identify login trigger, clicking last button")
                page.locator("header button").last.click()
            else:
                login_trigger.click()

            print("Waiting for AuthModal")
            auth_dialog = page.get_by_role("dialog")
            auth_dialog.wait_for()

            print("Filling login form")
            # Use placeholders as labels might not be linked
            page.get_by_placeholder("name@example.com").fill("user@example.com")
            # Password placeholder is •••••••• (8 dots).
            # It might vary. Let's use type="password"
            page.locator("input[type='password']").first.fill("password")

            print("Submitting login")
            auth_dialog.get_by_role("button", name="ログイン").click()

            # Wait for login to complete
            print("Waiting for AuthModal to close")
            auth_dialog.wait_for(state="hidden")

            # Now find the user menu button.
            print("Clicking user menu")

            # Re-find the button
            buttons = page.locator("header button").all()
            user_menu_btn = None
            for btn in buttons:
                label = btn.get_attribute("aria-label")
                if label == "検索を開く":
                    continue
                user_menu_btn = btn

            if user_menu_btn:
                user_menu_btn.click()
            else:
                page.locator("header button").last.click()

            # Click Profile menu item
            print("Clicking Profile menu item")
            # The menu item is a button inside the dropdown
            # We can search by text "プロフィール"
            page.get_by_role("button", name="プロフィール").click()

            # Verify Modal
            print("Verifying Modal")
            profile_dialog = page.get_by_role("dialog")
            profile_dialog.wait_for()

            if not profile_dialog.get_by_text("設定").is_visible():
                raise Exception("Profile modal title '設定' not found")

            # Switch Tabs
            print("Switching tabs")
            profile_dialog.get_by_role("tab", name="セキュリティ").click()
            # Replaced time.sleep with wait_for on the expected element
            profile_dialog.get_by_text("現在のパスワード").wait_for()

            profile_dialog.get_by_role("tab", name="連携").click()
            profile_dialog.get_by_text("GitHub").wait_for()

            profile_dialog.get_by_role("tab", name="パスキー").click()
            profile_dialog.get_by_text("パスキーを使用して").wait_for()

            profile_dialog.get_by_role("tab", name="プロフィール").click()
            profile_dialog.get_by_label("ユーザー名").wait_for()

            # Verify username field content
            if not profile_dialog.locator("input[value='Test User']").is_visible():
                 raise Exception("Username field not found or value incorrect")

            print("Taking screenshot")
            page.screenshot(path="verification/profile_modal.png")

        except Exception as e:
            print(f"Test failed: {e}")
            try:
                page.screenshot(path="verification/failure.png")
            except:
                pass
            raise e
        finally:
            browser.close()

if __name__ == "__main__":
    test_profile_modal()
