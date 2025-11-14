package com.zenowallet.photo

import android.content.ContentResolver
import android.content.Context
import android.net.Uri
import android.os.Build
import android.provider.MediaStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import java.io.IOException

class PhotoPlugin(private val context: Context) {

    private val allowedExt = arrayOf("jpg", "jpeg", "png")

    // 列出图片（只按文件名，不关心子目录）
    suspend fun listImages(): JSONArray = withContext(Dispatchers.IO) {
        val images = JSONArray()
        val projection = arrayOf(
            MediaStore.Images.Media.DISPLAY_NAME,
            MediaStore.Images.Media._ID
        )
        val sortOrder = "${MediaStore.Images.Media.DATE_ADDED} DESC"

        context.contentResolver.query(
            MediaStore.Images.Media.EXTERNAL_CONTENT_URI,
            projection,
            null,
            null,
            sortOrder
        )?.use { cursor ->
            val nameIndex = cursor.getColumnIndexOrThrow(MediaStore.Images.Media.DISPLAY_NAME)
            val idIndex = cursor.getColumnIndexOrThrow(MediaStore.Images.Media._ID)

            while (cursor.moveToNext()) {
                val name = cursor.getString(nameIndex)
                val id = cursor.getLong(idIndex)
                val ext = name.substringAfterLast('.', "").lowercase()
                if (!allowedExt.contains(ext)) continue

                val json = JSONObject()
                json.put("name", name)
                json.put("id", id) // 保留 id，用于按文件名读取
                images.put(json)
            }
        }
        images
    }

    // 按文件名读取单张图片 bytes
    suspend fun readImageByName(name: String): JSONObject = withContext(Dispatchers.IO) {
        val resolver: ContentResolver = context.contentResolver
        val projection = arrayOf(
            MediaStore.Images.Media._ID,
            MediaStore.Images.Media.DISPLAY_NAME
        )
        val selection = "${MediaStore.Images.Media.DISPLAY_NAME} = ?"
        val selectionArgs = arrayOf(name)

        resolver.query(
            MediaStore.Images.Media.EXTERNAL_CONTENT_URI,
            projection,
            selection,
            selectionArgs,
            null
        )?.use { cursor ->
            if (cursor.moveToFirst()) {
                val idIndex = cursor.getColumnIndexOrThrow(MediaStore.Images.Media._ID)
                val id = cursor.getLong(idIndex)
                val uri = Uri.withAppendedPath(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, id.toString())
                val bytes = readBytesFromUri(uri)
                return@withContext JSONObject().apply {
                    put("name", name)
                    put("bytes", bytes.map { it.toInt() and 0xFF })
                }
            }
        }

        throw IOException("Image not found: $name")
    }

    // 保存图片到系统相册
    suspend fun saveToGallery(bytes: ByteArray): JSONObject = withContext(Dispatchers.IO) {
        val contentValues = android.content.ContentValues().apply {
            put(MediaStore.Images.Media.DISPLAY_NAME, "image_${System.currentTimeMillis()}.png")
            put(MediaStore.Images.Media.MIME_TYPE, "image/png")
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                put(MediaStore.Images.Media.IS_PENDING, 1)
            }
        }

        val uri = context.contentResolver.insert(
            MediaStore.Images.Media.EXTERNAL_CONTENT_URI,
            contentValues
        ) ?: throw IOException("Failed to create new MediaStore record.")

        context.contentResolver.openOutputStream(uri).use { out ->
            if (out == null) throw IOException("Failed to get output stream.")
            out.write(bytes)
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            contentValues.clear()
            contentValues.put(MediaStore.Images.Media.IS_PENDING, 0)
            context.contentResolver.update(uri, contentValues, null, null)
        }

        JSONObject().apply { put("name", uri.lastPathSegment) }
    }

    // 辅助函数：Uri -> ByteArray
    private fun readBytesFromUri(uri: Uri): ByteArray {
        context.contentResolver.openInputStream(uri).use { input ->
            return input?.readBytes() ?: throw IOException("Failed to read from URI")
        }
    }
}
