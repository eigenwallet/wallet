-- Maps watchtowers to swaps
CREATE TABLE watchtower_peer_ids (
    swap_id TEXT PRIMARY KEY NOT NULL,
    peer_id TEXT NOT NULL
);
