source_name TEXT PRIMARY KEY,
last_post_id BIGINT NOT NULL,
updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);