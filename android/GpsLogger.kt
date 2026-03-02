package world.zerox1.node

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.location.Location
import android.os.Looper
import android.util.Log
import androidx.core.content.ContextCompat
import com.google.android.gms.location.*
import java.security.MessageDigest

/**
 * GpsLogger — passive background GPS collector for ZK-PoX.
 *
 * Records a signed GPS point every [intervalMs] milliseconds into an
 * encrypted local database. Each point is signed with the agent's
 * Ed25519 identity key so proofs can reference authenticated data.
 *
 * Integration:
 *   Copy to mobile/android/app/src/main/java/world/zerox1/node/GpsLogger.kt
 *   Add to build.gradle: implementation("com.google.android.gms:play-services-location:21.3.0")
 *   Start from NodeService.onCreate():
 *     gpsLogger = GpsLogger(applicationContext, identityKeyBytes, GpsDatabase(applicationContext))
 *     gpsLogger.start()
 */
class GpsLogger(
    private val context: Context,
    private val identityKey: ByteArray,
    private val db: GpsDatabase,
    private val intervalMs: Long = 5 * 60 * 1000L, // 5 minutes
) {
    companion object {
        private const val TAG = "GpsLogger"
        private const val RETENTION_DAYS = 365L
    }

    private var fusedClient: FusedLocationProviderClient? = null
    private var callback: LocationCallback? = null

    fun start() {
        if (!hasLocationPermission()) {
            Log.w(TAG, "Missing location permission — GPS logger not started.")
            return
        }

        fusedClient = LocationServices.getFusedLocationProviderClient(context)

        val request = LocationRequest.Builder(Priority.PRIORITY_BALANCED_POWER_ACCURACY, intervalMs)
            .setMinUpdateIntervalMillis(intervalMs / 2)
            .setWaitForAccurateLocation(false)
            .build()

        callback = object : LocationCallback() {
            override fun onLocationResult(result: LocationResult) {
                val loc = result.lastLocation ?: return
                recordPoint(loc)
            }
        }

        try {
            fusedClient?.requestLocationUpdates(request, callback!!, Looper.getMainLooper())
            Log.i(TAG, "GPS logger started (interval=${intervalMs / 1000}s)")
        } catch (e: SecurityException) {
            Log.e(TAG, "SecurityException starting GPS logger: $e")
        }

        pruneOldPoints()
    }

    fun stop() {
        callback?.let { fusedClient?.removeLocationUpdates(it) }
        callback = null
        fusedClient = null
        Log.i(TAG, "GPS logger stopped.")
    }

    private fun recordPoint(loc: Location) {
        val lat = loc.latitude
        val lng = loc.longitude
        val accuracy = loc.accuracy
        val timestamp = loc.time / 1000 // seconds

        val signature = signPoint(lat, lng, timestamp)

        db.insertPoint(lat, lng, accuracy, timestamp, signature)

        Log.d(TAG, "GPS point recorded: ($lat, $lng) accuracy=${accuracy}m")
    }

    /**
     * Sign SHA-256(lat_le || lng_le || timestamp_le) with the identity key
     * using Ed25519. The identity key must be 64 bytes (Ed25519 expanded
     * secret key) or 32 bytes (Ed25519 seed).
     *
     * Uses BouncyCastle's Ed25519 signer which is available on Android via
     * the SpongyCastle provider or the built-in security provider on API 28+.
     * If Ed25519 is not available (older API levels), falls back to HMAC-SHA256.
     */
    private fun signPoint(lat: Double, lng: Double, timestamp: Long): ByteArray {
        val message = buildMessage(lat, lng, timestamp)

        return try {
            // Android API 33+ has native Ed25519 in KeyFactory
            // For older devices, use the first 32 bytes as the seed
            val seed = if (identityKey.size >= 64) identityKey.sliceArray(0 until 32) else identityKey
            val spec = java.security.spec.PKCS8EncodedKeySpec(wrapEd25519Seed(seed))
            val kf = java.security.KeyFactory.getInstance("Ed25519")
            val privKey = kf.generatePrivate(spec)
            val sig = java.security.Signature.getInstance("Ed25519")
            sig.initSign(privKey)
            sig.update(message)
            sig.sign()
        } catch (e: Exception) {
            // Fallback: HMAC-SHA256 for devices without Ed25519 support
            Log.w(TAG, "Ed25519 not available, using HMAC-SHA256 fallback: ${e.message}")
            val mac = javax.crypto.Mac.getInstance("HmacSHA256")
            val keySpec = javax.crypto.spec.SecretKeySpec(identityKey, "HmacSHA256")
            mac.init(keySpec)
            mac.doFinal(message)
        }
    }

    /**
     * Wrap a 32-byte Ed25519 seed into PKCS#8 DER format for KeyFactory.
     * Ed25519 PKCS#8 = ASN.1 SEQUENCE { algorithm OID, OCTET STRING { seed } }
     */
    private fun wrapEd25519Seed(seed: ByteArray): ByteArray {
        val prefix = byteArrayOf(
            0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06,
            0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20
        )
        return prefix + seed
    }

    private fun buildMessage(lat: Double, lng: Double, timestamp: Long): ByteArray {
        val md = MessageDigest.getInstance("SHA-256")
        md.update(java.nio.ByteBuffer.allocate(8).order(java.nio.ByteOrder.LITTLE_ENDIAN).putDouble(lat).array())
        md.update(java.nio.ByteBuffer.allocate(8).order(java.nio.ByteOrder.LITTLE_ENDIAN).putDouble(lng).array())
        md.update(java.nio.ByteBuffer.allocate(8).order(java.nio.ByteOrder.LITTLE_ENDIAN).putLong(timestamp).array())
        return md.digest()
    }

    private fun pruneOldPoints() {
        val cutoff = System.currentTimeMillis() / 1000 - RETENTION_DAYS * 86_400
        val deleted = db.pruneOlderThan(cutoff)
        if (deleted > 0) {
            Log.i(TAG, "Pruned $deleted GPS points older than $RETENTION_DAYS days")
        }
    }

    private fun hasLocationPermission(): Boolean =
        ContextCompat.checkSelfPermission(context, Manifest.permission.ACCESS_FINE_LOCATION) ==
            PackageManager.PERMISSION_GRANTED ||
        ContextCompat.checkSelfPermission(context, Manifest.permission.ACCESS_COARSE_LOCATION) ==
            PackageManager.PERMISSION_GRANTED
}
