CREATE TABLE IF NOT EXISTS sftp_config (
                                           config_id INTEGER PRIMARY KEY,
                                           host TEXT NOT NULL,
                                           port INTEGER DEFAULT 22,
                                           username TEXT NOT NULL,
                                           password TEXT
);