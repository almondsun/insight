use rusqlite::Connection;

fn error(error: rusqlite::Error) -> String {
    error.to_string()
}

pub fn migrate(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "BEGIN IMMEDIATE;
             CREATE TABLE IF NOT EXISTS fame_observations(
               id INTEGER PRIMARY KEY,
               norm TEXT NOT NULL,
               followers INTEGER NOT NULL CHECK(followers >= 0),
               following INTEGER NOT NULL CHECK(following >= 0),
               precision TEXT NOT NULL CHECK(precision IN ('exact','approximate')),
               observed_at TEXT NOT NULL,
               source TEXT NOT NULL,
               corpus_release TEXT NOT NULL,
               authenticated INTEGER NOT NULL CHECK(authenticated=1),
               UNIQUE(id,norm,precision),
               UNIQUE(norm,observed_at,source,corpus_release)
             );
             CREATE TABLE IF NOT EXISTS fame_runs(
               id INTEGER PRIMARY KEY,
               snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
               created_at TEXT NOT NULL,
               completed_at TEXT,
               status TEXT NOT NULL CHECK(status IN ('pending','active','paused','completed','cancelled','blocked','failed')),
               formula_version TEXT NOT NULL,
               corpus_release TEXT,
               profile_version TEXT,
               refresh_all INTEGER NOT NULL DEFAULT 0 CHECK(refresh_all IN (0,1))
             );
             CREATE UNIQUE INDEX IF NOT EXISTS idx_one_active_fame_run ON fame_runs((1)) WHERE status='active';
             CREATE TABLE IF NOT EXISTS fame_run_members(
               run_id INTEGER NOT NULL REFERENCES fame_runs(id) ON DELETE CASCADE,
               norm TEXT NOT NULL,
               username TEXT NOT NULL,
               status TEXT NOT NULL CHECK(status IN ('pending','exact','approximate','private','missing','blocked','failed','cancelled')),
               observation_id INTEGER,
               PRIMARY KEY(run_id,norm),
               FOREIGN KEY(observation_id,norm,status) REFERENCES fame_observations(id,norm,precision) ON DELETE RESTRICT,
               CHECK((status IN ('exact','approximate') AND observation_id IS NOT NULL) OR (status NOT IN ('exact','approximate') AND observation_id IS NULL))
             );
             CREATE INDEX IF NOT EXISTS idx_fame_members_status ON fame_run_members(run_id,status);
             PRAGMA user_version=2;
             COMMIT;",
        )
        .map_err(error)
}

pub fn collect_unreferenced_observations(connection: &Connection) -> Result<(), String> {
    connection
        .execute(
            "DELETE FROM fame_observations
             WHERE NOT EXISTS (
               SELECT 1 FROM fame_run_members
               WHERE observation_id=fame_observations.id
             )",
            [],
        )
        .map_err(error)?;
    Ok(())
}
