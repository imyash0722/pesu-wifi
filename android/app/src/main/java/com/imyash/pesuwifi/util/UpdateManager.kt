package com.imyash.pesuwifi.util

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.core.content.FileProvider
import com.imyash.pesuwifi.BuildConfig
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.util.concurrent.TimeUnit

data class UpdateInfo(
    val tagName: String,
    val versionName: String,
    val releaseTitle: String,
    val releaseNotes: String,
    val downloadUrl: String,
    val apkName: String,
    val fileSize: Long,
    val publishedAt: String
)

sealed class UpdateState {
    object Idle : UpdateState()
    object Checking : UpdateState()
    data class Available(val updateInfo: UpdateInfo) : UpdateState()
    object UpToDate : UpdateState()
    data class Downloading(val progress: Float, val bytesDownloaded: Long, val totalBytes: Long) : UpdateState()
    data class ReadyToInstall(val apkFile: File) : UpdateState()
    data class Error(val message: String) : UpdateState()
}

object UpdateManager {
    private const val TAG = "UpdateManager"
    private const val GITHUB_RELEASES_API = "https://api.github.com/repos/imyash0722/pesu-wifi/releases/latest"

    private val _updateState = MutableStateFlow<UpdateState>(UpdateState.Idle)
    val updateState: StateFlow<UpdateState> = _updateState.asStateFlow()

    private val httpClient = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .build()

    private var lastCheckedTimestamp: Long = 0L

    fun resetState() {
        _updateState.value = UpdateState.Idle
    }

    /**
     * Checks GitHub API for the latest published release.
     * Compares semver against BuildConfig.VERSION_NAME.
     */
    suspend fun checkForUpdates(context: Context, force: Boolean = false): UpdateInfo? = withContext(Dispatchers.IO) {
        val now = System.currentTimeMillis()
        if (!force && now - lastCheckedTimestamp < 15 * 60 * 1000L && _updateState.value is UpdateState.Available) {
            return@withContext (_updateState.value as UpdateState.Available).updateInfo
        }

        _updateState.value = UpdateState.Checking
        AppLogger.i(TAG, "Checking for updates at $GITHUB_RELEASES_API...")

        try {
            val request = Request.Builder()
                .url(GITHUB_RELEASES_API)
                .header("Accept", "application/vnd.github.v3+json")
                .header("User-Agent", "PESU-WiFi-Android/${BuildConfig.VERSION_NAME}")
                .build()

            val response = httpClient.newCall(request).execute()
            if (!response.isSuccessful) {
                val err = "GitHub API returned HTTP ${response.code}"
                AppLogger.w(TAG, err)
                _updateState.value = UpdateState.Error(err)
                return@withContext null
            }

            val body = response.body?.string() ?: throw IllegalStateException("Empty response body")
            val json = JSONObject(body)

            val tagName = json.optString("tag_name", "").trim()
            val releaseTitle = json.optString("name", tagName)
            val releaseNotes = json.optString("body", "No release notes provided.")
            val publishedAt = json.optString("published_at", "")

            val cleanRemoteVersion = tagName.removePrefix("v").trim()
            val currentVersion = BuildConfig.VERSION_NAME.split("-")[0].removePrefix("v").trim()

            AppLogger.d(TAG, "Remote version: $cleanRemoteVersion, Current version: $currentVersion")

            if (isNewerVersion(cleanRemoteVersion, currentVersion)) {
                val assets = json.optJSONArray("assets")
                var selectedDownloadUrl = ""
                var selectedApkName = ""
                var selectedSize = 0L

                val isTesterFlavor = BuildConfig.FLAVOR.equals("beta", ignoreCase = true) ||
                        BuildConfig.VERSION_NAME.contains("tester", ignoreCase = true)
                val targetFlavorKeyword = if (isTesterFlavor) "tester" else "stable"

                if (assets != null) {
                    for (i in 0 until assets.length()) {
                        val asset = assets.getJSONObject(i)
                        val name = asset.optString("name", "")
                        if (name.endsWith(".apk", ignoreCase = true)) {
                            if (name.contains(targetFlavorKeyword, ignoreCase = true)) {
                                selectedDownloadUrl = asset.optString("browser_download_url", "")
                                selectedApkName = name
                                selectedSize = asset.optLong("size", 0L)
                                break
                            } else if (selectedDownloadUrl.isEmpty()) {
                                selectedDownloadUrl = asset.optString("browser_download_url", "")
                                selectedApkName = name
                                selectedSize = asset.optLong("size", 0L)
                            }
                        }
                    }
                }

                if (selectedDownloadUrl.isNotEmpty()) {
                    val info = UpdateInfo(
                        tagName = tagName,
                        versionName = cleanRemoteVersion,
                        releaseTitle = releaseTitle,
                        releaseNotes = releaseNotes,
                        downloadUrl = selectedDownloadUrl,
                        apkName = selectedApkName,
                        fileSize = selectedSize,
                        publishedAt = publishedAt
                    )
                    lastCheckedTimestamp = now
                    _updateState.value = UpdateState.Available(info)
                    AppLogger.i(TAG, "🎉 New update available: $cleanRemoteVersion ($selectedApkName)")
                    return@withContext info
                } else {
                    AppLogger.w(TAG, "Release $tagName found but no APK asset matched flavor '$targetFlavorKeyword'")
                    _updateState.value = UpdateState.UpToDate
                    return@withContext null
                }
            } else {
                lastCheckedTimestamp = now
                _updateState.value = UpdateState.UpToDate
                AppLogger.i(TAG, "App is up to date (v$currentVersion)")
                return@withContext null
            }
        } catch (e: Exception) {
            AppLogger.e(TAG, "Update check failed: ${e.message}", e)
            _updateState.value = UpdateState.Error(e.message ?: "Failed to check for updates")
            null
        }
    }

    /**
     * Downloads the APK file to the application cache directory.
     */
    suspend fun downloadUpdate(context: Context, updateInfo: UpdateInfo): File = withContext(Dispatchers.IO) {
        val updatesDir = File(context.cacheDir, "updates").apply { mkdirs() }
        val apkFile = File(updatesDir, updateInfo.apkName)

        _updateState.value = UpdateState.Downloading(0f, 0L, updateInfo.fileSize)
        AppLogger.i(TAG, "Downloading update from ${updateInfo.downloadUrl} to ${apkFile.absolutePath}...")

        val request = Request.Builder()
            .url(updateInfo.downloadUrl)
            .header("User-Agent", "PESU-WiFi-Android/${BuildConfig.VERSION_NAME}")
            .build()

        val response = httpClient.newCall(request).execute()
        if (!response.isSuccessful) {
            val err = "Download failed: HTTP ${response.code}"
            _updateState.value = UpdateState.Error(err)
            throw IllegalStateException(err)
        }

        val body = response.body ?: throw IllegalStateException("Empty download response body")
        val totalBytes = if (updateInfo.fileSize > 0) updateInfo.fileSize else body.contentLength()

        body.byteStream().use { input ->
            FileOutputStream(apkFile).use { output ->
                val buffer = ByteArray(8 * 1024)
                var bytesRead: Int
                var totalRead = 0L

                while (input.read(buffer).also { bytesRead = it } != -1) {
                    output.write(buffer, 0, bytesRead)
                    totalRead += bytesRead
                    val progress = if (totalBytes > 0) totalRead.toFloat() / totalBytes else 0f
                    _updateState.value = UpdateState.Downloading(progress, totalRead, totalBytes)
                }
                output.flush()
            }
        }

        AppLogger.i(TAG, "Update downloaded successfully (${apkFile.length()} bytes)")
        _updateState.value = UpdateState.ReadyToInstall(apkFile)
        apkFile
    }

    /**
     * Launches Android's native package installer to install the downloaded APK.
     */
    fun installUpdate(context: Context, apkFile: File): Boolean {
        if (!apkFile.exists() || apkFile.length() == 0L) {
            AppLogger.e(TAG, "Cannot install: APK file does not exist or is empty")
            _updateState.value = UpdateState.Error("Downloaded APK file is corrupt or missing")
            return false
        }

        // On Android 8.0+ (Oreo, API 26+), check unknown app sources permission
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            if (!context.packageManager.canRequestPackageInstalls()) {
                AppLogger.w(TAG, "Missing REQUEST_INSTALL_PACKAGES permission, opening settings...")
                val intent = Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES).apply {
                    data = Uri.parse("package:${context.packageName}")
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                }
                context.startActivity(intent)
                return false
            }
        }

        return try {
            val apkUri: Uri = FileProvider.getUriForFile(
                context,
                "${context.packageName}.fileprovider",
                apkFile
            )

            val installIntent = Intent(Intent.ACTION_VIEW).apply {
                setDataAndType(apkUri, "application/vnd.android.package-archive")
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP)
            }

            AppLogger.i(TAG, "Launching Android PackageInstaller for URI: $apkUri")
            context.startActivity(installIntent)
            true
        } catch (e: Exception) {
            AppLogger.e(TAG, "Failed to launch package installer: ${e.message}", e)
            _updateState.value = UpdateState.Error("Failed to launch package installer: ${e.message}")
            false
        }
    }

    /**
     * Compares two semantic version strings (e.g. "3.8.0" vs "3.7.0").
     * Returns true if remote is strictly greater than current.
     */
    fun isNewerVersion(remote: String, current: String): Boolean {
        try {
            val remoteParts = remote.split(".").mapNotNull { it.toIntOrNull() }
            val currentParts = current.split(".").mapNotNull { it.toIntOrNull() }

            val maxLen = maxOf(remoteParts.size, currentParts.size)
            for (i in 0 until maxLen) {
                val r = remoteParts.getOrElse(i) { 0 }
                val c = currentParts.getOrElse(i) { 0 }
                if (r > c) return true
                if (r < c) return false
            }
            return false
        } catch (e: Exception) {
            return false
        }
    }
}
