# Database TABLE
```sql
CREATE TABLE IF NOT EXISTS table_name (
    _id TEXT PRIMARY KEY,       -- BSON ObjectId
    data JSONB NOT NULL,        -- JSON
    _created_at TIMESTAMPTZ DEFAULT NOW(),
    _updated_at TIMESTAMPTZ DEFAULT NOW()
);
```