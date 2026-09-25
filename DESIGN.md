
## Block model

### Schema of blocks table

```sql
CREATE TABLE blocks (
  id         TEXT PRIMARY KEY,            -- uuid
  parent_id  TEXT REFERENCES blocks(id),  -- NULL = workspace root
  type       TEXT NOT NULL,               -- 'page' | 'text' | 'heading1' | ...
  props      TEXT NOT NULL DEFAULT '{}',  -- JSON, per-type (todo: {checked})
  position   REAL NOT NULL,               -- sibling order
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX idx_parent ON blocks(parent_id, position);
```

There should be an ON DELETE CASCADE config of the `blocks` table so that when a parent block is deleted, all its children are deleted as well. I should not have orphan blocks. 

### Design of 'props'

For the `props` of a block. I am chosing to use Typed Enum `BlockType`. An alternative would have been to use a schemaless JSON object. I prefer I typed enum because the validity of a document can be enforced at compile time. The inconvenience is that this make it harder to add a new type because I would need to update the database schema as well.

### Serialization

- the uuid crate needs to have the `serde` feature flag enabled for Uuids to be serializable
