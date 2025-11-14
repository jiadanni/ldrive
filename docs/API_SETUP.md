# Google Drive API Setup Guide

This guide will walk you through setting up Google Drive API credentials for DriveSync.

## Overview

To use DriveSync, you need to create OAuth 2.0 credentials in Google Cloud Console. These credentials allow DriveSync to access your Google Drive on your behalf.

## Step-by-Step Instructions

### 1. Create a Google Cloud Project

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Click "Select a project" at the top
3. Click "New Project"
4. Enter project name: "DriveSync" (or any name you prefer)
5. Click "Create"

### 2. Enable Google Drive API

1. In your project, go to **APIs & Services > Library**
2. Search for "Google Drive API"
3. Click on "Google Drive API"
4. Click "Enable"

### 3. Configure OAuth Consent Screen

1. Go to **APIs & Services > OAuth consent screen**
2. Select **External** (unless you have Google Workspace)
3. Click "Create"
4. Fill in required fields:
   - **App name**: DriveSync
   - **User support email**: Your email
   - **Developer contact information**: Your email
5. Click "Save and Continue"
6. On "Scopes" page, click "Add or Remove Scopes"
7. Add these scopes:
   - `.../auth/drive.file` - View and manage Google Drive files
   - `.../auth/drive.metadata.readonly` - View metadata
   - `.../auth/userinfo.email` - View email address
8. Click "Update" then "Save and Continue"
9. On "Test users" page (if in testing mode):
   - Click "Add Users"
   - Add your email address
   - Click "Save and Continue"
10. Review and click "Back to Dashboard"

### 4. Create OAuth 2.0 Credentials

1. Go to **APIs & Services > Credentials**
2. Click "Create Credentials" > "OAuth client ID"
3. Select application type: **Desktop app**
4. Enter name: "DriveSync Desktop Client"
5. Click "Create"
6. A dialog will show your credentials:
   - **Client ID**: Starts with `xxxxxx.apps.googleusercontent.com`
   - **Client Secret**: Random string
7. **IMPORTANT**: Copy both values - you'll need them!
8. Click "OK"

### 5. Download Credentials (Optional)

You can download the credentials as JSON:

1. On the Credentials page, find your OAuth 2.0 Client ID
2. Click the download button (⬇️)
3. Save as `client_secret.json`

The JSON format:
```json
{
  "installed": {
    "client_id": "YOUR_CLIENT_ID.apps.googleusercontent.com",
    "client_secret": "YOUR_CLIENT_SECRET",
    "redirect_uris": ["http://localhost"]
  }
}
```

## Using Your Credentials

### Method 1: Environment Variables (Recommended)

```bash
export GOOGLE_CLIENT_ID="your_client_id.apps.googleusercontent.com"
export GOOGLE_CLIENT_SECRET="your_client_secret"
```

Add to your `~/.bashrc` or `~/.zshrc` for persistence:
```bash
echo 'export GOOGLE_CLIENT_ID="your_client_id"' >> ~/.bashrc
echo 'export GOOGLE_CLIENT_SECRET="your_client_secret"' >> ~/.bashrc
source ~/.bashrc
```

### Method 2: Configuration File

Create `~/.config/drivesync/credentials.toml`:

```toml
[oauth]
client_id = "your_client_id.apps.googleusercontent.com"
client_secret = "your_client_secret"
```

**SECURITY NOTE**: Make sure this file is only readable by you:
```bash
chmod 600 ~/.config/drivesync/credentials.toml
```

### Method 3: Command Line Arguments

Pass credentials directly when running examples:

```bash
cargo run --example oauth_flow -- "your_client_id" "your_client_secret"
```

## Testing Your Setup

### 1. Run OAuth Flow Example

```bash
# Using environment variables
cargo run --example oauth_flow -- "$GOOGLE_CLIENT_ID" "$GOOGLE_CLIENT_SECRET"

# Or with direct values
cargo run --example oauth_flow -- "your_client_id" "your_client_secret"
```

This will:
1. Generate an authorization URL
2. Open your browser for authentication
3. Capture the authorization code
4. Exchange it for access tokens
5. Store credentials securely in system keyring

### 2. List Your Drive Files

After authentication:

```bash
cargo run --example list_files -- "your_email@gmail.com" "$GOOGLE_CLIENT_ID" "$GOOGLE_CLIENT_SECRET"
```

### 3. Upload a Test File

```bash
# Create a test file
echo "Hello, Google Drive!" > test.txt

# Upload it
cargo run --example upload_file -- "your_email@gmail.com" "$GOOGLE_CLIENT_ID" "$GOOGLE_CLIENT_SECRET" test.txt
```

## Security Best Practices

### DO:
- ✅ Store credentials in environment variables or secure config files
- ✅ Set restrictive file permissions (chmod 600)
- ✅ Use the system keyring for OAuth tokens
- ✅ Never commit credentials to version control
- ✅ Use different credentials for development and production

### DON'T:
- ❌ Hard-code credentials in source code
- ❌ Share credentials publicly
- ❌ Commit `client_secret.json` to git
- ❌ Store access tokens in plain text files
- ❌ Use production credentials for testing

## Troubleshooting

### "Access blocked: This app's request is invalid"

**Solution**: Make sure you've configured the OAuth consent screen properly and added your email as a test user.

### "Error 400: redirect_uri_mismatch"

**Solution**: The redirect URI in your code must match what's configured in Google Cloud Console. DriveSync uses `http://localhost:8080`.

To fix:
1. Go to **APIs & Services > Credentials**
2. Edit your OAuth 2.0 Client ID
3. Add `http://localhost:8080` to "Authorized redirect URIs"
4. Click "Save"

### "Error 403: Access forbidden"

**Solution**: Make sure you've enabled the Google Drive API for your project.

### "Keyring error" on Linux

**Solution**: Install and configure gnome-keyring or another secret service:

```bash
# Ubuntu/Debian
sudo apt install gnome-keyring

# Arch Linux
sudo pacman -S gnome-keyring

# Start the keyring daemon
gnome-keyring-daemon --start
```

### Rate Limit Exceeded

**Solution**: Google Drive API has usage quotas. If you hit limits:
- Wait a few minutes before retrying
- Check your quota usage in Google Cloud Console
- Consider requesting quota increase if needed

## API Quotas and Limits

Default quotas (as of 2024):
- **Queries per day**: 1,000,000,000
- **Queries per 100 seconds per user**: 1,000
- **Upload limit**: 750 GB per day

For most users, these limits are more than sufficient. Heavy users can request quota increases in Google Cloud Console.

## Next Steps

After setting up your credentials:

1. Read the [User Guide](USER_GUIDE.md) for day-to-day usage
2. Check the [Developer Guide](DEVELOPER.md) for API details
3. Explore the examples in `examples/` directory
4. Run DriveSync with `cargo run`

## Additional Resources

- [Google Drive API Documentation](https://developers.google.com/drive/api/v3/about-sdk)
- [OAuth 2.0 Documentation](https://developers.google.com/identity/protocols/oauth2)
- [Google Cloud Console](https://console.cloud.google.com/)
- [DriveSync GitHub Issues](https://github.com/jiadanni/ldrive/issues)

## Need Help?

If you encounter issues:
1. Check this guide carefully
2. Review the error messages
3. Search [existing issues](https://github.com/jiadanni/ldrive/issues)
4. Create a new issue with details (but never include your credentials!)

---

**Remember**: Keep your credentials secure! Never share your `client_secret` or access tokens publicly.
