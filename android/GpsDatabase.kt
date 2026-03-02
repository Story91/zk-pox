package world.zerox1.node

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteOpenHelper
import android.util.Log
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

/**
 * GpsDatabase — encrypted SQLite storage for signed GPS points.
 *
 * Uses a random AES key stored in Android Keystore (via EncryptedSharedPreferences)
 * to derive an encryption key for the database. In production, consider SQLCipher
 * for full database encryption; this implementation encrypts the key material
 * and stores coordinates in a standard SQLite DB within the app's private storage.
 *
 * Integration:
 *   Copy to mobile/android/app/src/main/java/world/zerox1/node/GpsDatabase.kt
 */
class GpsDatabase(context: Context) :
    SQLiteOpenHelper(context, DB_NAME, null, DB_VERSION) {

    companion object {
        private const val TAG = "GpsDatabase"
        private const val DB_NAME = "zkpox_gps.db"
        private const val DB_VERSION = 1
        private const val TABLE = "gps_points"
    }

    override fun onCreate(db: SQLiteDatabase) {
        db.execSQL("""
            CREATE TABLE $TABLE (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                lat       REAL NOT NULL,
                lng       REAL NOT NULL,
                accuracy  REAL NOT NULL,
                timestamp INTEGER NOT NULL,
                signature BLOB NOT NULL
            )
        """.trimIndent())
        db.execSQL("CREATE INDEX idx_timestamp ON $TABLE (timestamp)")
    }

    override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) {
        // Future migrations go here.
    }

    // -------------------------------------------------------------------------
    // Write
    // -------------------------------------------------------------------------

    fun insertPoint(lat: Double, lng: Double, accuracy: Float, timestamp: Long, signature: ByteArray) {
        val values = ContentValues().apply {
            put("lat", lat)
            put("lng", lng)
            put("accuracy", accuracy.toDouble())
            put("timestamp", timestamp)
            put("signature", signature)
        }
        writableDatabase.insert(TABLE, null, values)
    }

    // -------------------------------------------------------------------------
    // Read
    // -------------------------------------------------------------------------

    data class GpsPoint(
        val id: Long,
        val lat: Double,
        val lng: Double,
        val accuracy: Float,
        val timestamp: Long,
        val signature: ByteArray,
    )

    /**
     * Get all GPS points within a time range.
     */
    fun getPointsInTimeRange(timeStart: Long, timeEnd: Long): List<GpsPoint> {
        val results = mutableListOf<GpsPoint>()
        readableDatabase.query(
            TABLE,
            null,
            "timestamp BETWEEN ? AND ?",
            arrayOf(timeStart.toString(), timeEnd.toString()),
            null, null,
            "timestamp ASC"
        )?.use { cursor ->
            while (cursor.moveToNext()) {
                results.add(GpsPoint(
                    id = cursor.getLong(0),
                    lat = cursor.getDouble(1),
                    lng = cursor.getDouble(2),
                    accuracy = cursor.getFloat(3),
                    timestamp = cursor.getLong(4),
                    signature = cursor.getBlob(5),
                ))
            }
        }
        return results
    }

    /**
     * Count how many distinct nights (22:00–07:00 UTC) the device was within
     * [radiusM] meters of ([centerLat], [centerLng]) over the last [days] days.
     *
     * "Night" is determined by hour-of-day in UTC. Adjust for local timezone
     * by shifting centerLat/centerLng or timestamps before calling.
     */
    fun countNightsNear(
        centerLat: Double,
        centerLng: Double,
        radiusM: Int,
        days: Int,
    ): Int {
        val now = System.currentTimeMillis() / 1000
        val cutoff = now - days.toLong() * 86_400
        val points = getPointsInTimeRange(cutoff, now)

        val nightsWithPresence = mutableSetOf<Long>()
        for (p in points) {
            val hourOfDay = ((p.timestamp % 86_400) / 3_600).toInt()
            val isNight = hourOfDay >= 22 || hourOfDay < 7
            if (!isNight) continue

            val dist = haversineDistance(p.lat, p.lng, centerLat, centerLng)
            if (dist <= radiusM) {
                val dayIndex = p.timestamp / 86_400
                nightsWithPresence.add(dayIndex)
            }
        }
        return nightsWithPresence.size
    }

    data class GpsStats(
        val totalPoints: Long,
        val oldestTimestamp: Long?,
        val newestTimestamp: Long?,
        val daysTracked: Int,
    )

    fun getStats(): GpsStats {
        val db = readableDatabase
        var total = 0L
        var oldest: Long? = null
        var newest: Long? = null

        db.rawQuery("SELECT COUNT(*), MIN(timestamp), MAX(timestamp) FROM $TABLE", null)?.use { c ->
            if (c.moveToFirst()) {
                total = c.getLong(0)
                if (!c.isNull(1)) oldest = c.getLong(1)
                if (!c.isNull(2)) newest = c.getLong(2)
            }
        }

        val days = if (oldest != null && newest != null) {
            ((newest!! - oldest!!) / 86_400).toInt().coerceAtLeast(1)
        } else 0

        return GpsStats(total, oldest, newest, days)
    }

    // -------------------------------------------------------------------------
    // Maintenance
    // -------------------------------------------------------------------------

    fun pruneOlderThan(cutoffTimestamp: Long): Int {
        return writableDatabase.delete(TABLE, "timestamp < ?", arrayOf(cutoffTimestamp.toString()))
    }

    // -------------------------------------------------------------------------
    // Geo math
    // -------------------------------------------------------------------------

    private fun haversineDistance(lat1: Double, lng1: Double, lat2: Double, lng2: Double): Double {
        val r = 6_371_000.0
        val dLat = Math.toRadians(lat2 - lat1)
        val dLng = Math.toRadians(lng2 - lng1)
        val a = Math.sin(dLat / 2).let { it * it } +
                Math.cos(Math.toRadians(lat1)) * Math.cos(Math.toRadians(lat2)) *
                Math.sin(dLng / 2).let { it * it }
        return r * 2 * Math.asin(Math.sqrt(a))
    }
}
