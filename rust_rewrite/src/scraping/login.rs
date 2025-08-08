use fantoccini::{Client, Locator};
use crate::error::AppError;
use crate::config::AmazonConfig;

pub enum LoginMethod {
    Manual,
    Automated { email: String, password: String },
}

pub async fn handle_login(
    client: &Client,
    method: LoginMethod,
) -> Result<(), AppError> {
    match method {
        LoginMethod::Manual => manual_login(client).await,
        LoginMethod::Automated { email, password } => {
            automated_login(client, &email, &password).await
        }
    }
}

async fn manual_login(client: &Client) -> Result<(), AppError> {
    // Navigate to Prime Video watch history
    client
        .goto("https://www.primevideo.com/settings/watch-history")
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    // Wait for user to manually login
    tokio::time::sleep(std::time::Duration::from_secs(300)).await; // 5 minute timeout

    // Verify login success
    if !is_logged_in(client).await? {
        return Err(AppError::AuthError("Manual login failed".into()));
    }

    Ok(())
}

async fn automated_login(
    client: &Client,
    email: &str,
    password: &str,
) -> Result<(), AppError> {
    // Navigate to Amazon login
    client
        .goto("https://www.amazon.com/ap/signin")
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    // Fill email
    client
        .find(Locator::Id("ap_email"))
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?
        .send_keys(email)
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    // Click continue
    client
        .find(Locator::Id("continue"))
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?
        .click()
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    // Fill password
    client
        .find(Locator::Id("ap_password"))
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?
        .send_keys(password)
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    // Submit
    client
        .find(Locator::Id("signInSubmit"))
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?
        .click()
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    // Handle 2FA if present
    if let Ok(element) = client
        .find(Locator::Css("#auth-mfa-otpcode, .cvf-widget-input-code"))
        .await
    {
        return Err(AppError::AuthError(
            "2FA detected - manual login required".into(),
        ));
    }

    // Verify login success
    if !is_logged_in(client).await? {
        return Err(AppError::AuthError("Automated login failed".into()));
    }

    Ok(())
}

async fn is_logged_in(client: &Client) -> Result<bool, AppError> {
    let current_url = client
        .current_url()
        .await
        .map_err(|e| AppError::BrowserError(e.to_string()))?;

    Ok(current_url.contains("watch-history") && 
       !current_url.contains("signin") && 
       !current_url.contains("auth"))
}