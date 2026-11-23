package br.acerola.comic.local.dao.sync

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import br.acerola.comic.local.entity.sync.SyncHistoryLog

@Dao
interface SyncHistoryLogDao {
    @Insert
    suspend fun insert(entry: SyncHistoryLog)

    /** Most recent first — used to populate the activity log when opening the sync screen. */
    @Query("SELECT * FROM sync_history_log ORDER BY created_at DESC, id DESC LIMIT :limit")
    suspend fun findRecent(limit: Int): List<SyncHistoryLog>

    /** "Clear" button on the Network screen — same operation as Desktop (`SyncHistoryLogRepository::delete_all`). */
    @Query("DELETE FROM sync_history_log")
    suspend fun deleteAll()
}
