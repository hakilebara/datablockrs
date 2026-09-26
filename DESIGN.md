
## Block model

### Schema of blocks table

```
This schema will require to set `PRAGMA foreign_keys = ON;` to enable the foreign key feature of sqlite

Note: a FOREIGN KEY constraint ignores NULLs - meaning if the foreign key column ofr an entry in the child table is NULL, the FK constraint is still valid


// table 'blocks'
id TEXT PRIMARY KEY - stored as text because SQLite has no UUDI type. 


parent_id TEXT REFERENCES blocks(id) ON DELETE CASCADE- if NULL then block is the root block

position REAL - when a new block is inserted between two existing adjacent blocks, the position of the new block should be the average of the position of the adjacent blocks.

props TEXT - json blob of the block properties eg. { checked: false } for a 'todo'

type TEXT NOT NULL CHECK (type IN ('page', 'text', 'todo','heading1', 'heading2', 'heading3', 'bulleted_list_item', 'numbered_list_item', 'quote', 'divider', 'code')) - the block type

created_at TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL - by default it will set the cell to the current timestamp at the creation of the row

updated_at TEXT - similar to 'created_at', but can be NULL will be set by an AFTER UPDATE SQLite trigger

```

### Fractional indexing

With each block having an index. Inserting a block between two adjacent blocks would require a re-ordering of the blocks ids - if the block ids where ints. Fractional indexing is a strategy that aims at minimizing the amount of operations needed to insert a block between two neighbors. Note: This only applies to blocks that share the same parent.

### Design of 'type'

`type` is a column in the `blocks` table its purpose is to list all the possible blocks types of the system. A CHECK constraints ensure that no invalid block type can be inserted in the database. The inconvenience of this approach is that every time a new type is added, the database schema will have to change. This is OK as I don't plan on doing that often. 

### Design of 'props'

The props column only stores the payload of a block. The type column stores the variant of the block.

| type                 | props                                          |
|----------------------|------------------------------------------------|
| todo                 | {"checked": false, "text": "do it"}            |
| page                 | {"title": "Hello World"}                       |
| code                 | {"language": "Python", "content": "some code"} |
| text                 | {"text": "This is some text"}                  |
| divider              | {}                                             |
| header1              | {"text": "This is a h1 title"}                 |
| header2              | {"text": "This is a h2 title"}                 |
| header3              | {"text": "This is a h2 title"}                 |
| quote                | {"text": "quoted text"}                        |
| bulleted_list_item   | {"text": "this is a bullet item"}              |
| numbered_list_item   | {"text": "this is a numbered item"}            |

`props` is stored as a JSON blob in SQLite. The challenge is how do I make sure that a given prop is valid and is compatible with the `type` column of the same row.

### Indexing the `blocks` table

The most common sql command is going to be : find the blocks whose parent_id is this. Namely, find all the children of a given block. The SQL will most likely look like this : `... WHERE parent_id = <id> ORDER BY position;`

```sql
CREATE INDEX idx_blocks_parent ON blocks (parent_id, position);
```

Therefore it makes sense to add an INDEX on `parent_id`.

### Serialization

- the uuid crate needs to have the `serde` feature flag enabled for Uuids to be serializable

Example of serialization of a Todo block

```json
{
  "type": "todo",
  "id": "c02fc1d3-db8b-45c5-a222-27595b15aea7",
  "parent": "59833787-2cf9-4fdf-8782-e53db20768a5",
  "created_at": "2026-03-01T19:05:00.000Z",
  "updated_at": "2026-07-06T19:41:00.000Z",
  "todo": {
      "text": "do it",
      "checked": "false",
  }
}
```
