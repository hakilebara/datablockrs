
## Block model

### Schema of blocks table

```
This schema will require to set `PRAGMA foreign_keys = ON;` to enable the foreign key feature of sqlite

Note: a FOREIGN KEY constraint ignores NULLs - meaning if the foreign key column of an entry in the child table is NULL, the FK constraint is still valid


// table 'blocks'
id TEXT PRIMARY KEY - stored as text because SQLite has no UUID type. 


parent_id TEXT REFERENCES blocks(id) ON DELETE CASCADE- if NULL then block is the root block

position REAL - when a new block is inserted between two existing adjacent blocks, the position of the new block should be the average of the position of the adjacent blocks.

props TEXT - json blob of the block properties eg. { checked: false } for a 'todo'

type TEXT NOT NULL - the block type

created_at TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL - by default it will set the cell to the current timestamp at the creation of the row

updated_at TEXT - similar to 'created_at', but can be NULL will be set by an AFTER UPDATE SQLite trigger

```

### Fractional indexing

With each block having an index. Inserting a block between two adjacent blocks would require a re-ordering of the blocks ids - if the block ids where ints. Fractional indexing is a strategy that aims at minimizing the amount of operations needed to insert a block between two neighbors. Note: This only applies to blocks that share the same parent.

### Design of 'type'

`type` is a column in the `blocks` table its purpose is to list all the possible blocks types of the system.
At first I considered using a CHECK constraints to ensure no invalid block type could be inserted in the database. The inconvenience of this approach is that every time a new type is added, the database needs to be destroyed and rebuild with the new CHECK constraint. This is such a significant migration cost that I won't consider it. 
The validity constraint on type will be enforced in code. 

#### Adding a new type

Let say I want to add a new block type `Callout`:
- add a new variant in the `BlockType` enum:
    ```
    Callout { text: String }
    ```

### Design of 'props'

The props column only stores the payload of a block. The type column stores the variant of the block.

| type                 | props                                          |
|----------------------|------------------------------------------------|
| todo                 | {"checked": false, "text": "do it"}            |
| page                 | {"title": "Hello World"}                       |
| code                 | {"language": "Python", "content": "some code"} |
| text                 | {"text": "This is some text"}                  |
| divider              | {}                                             |
| heading1             | {"text": "This is a h1 title"}                 |
| heading2             | {"text": "This is a h2 title"}                 |
| heading3             | {"text": "This is a h3 title"}                 |
| quote                | {"text": "quoted text"}                        |
| bulleted_list_item   | {"text": "this is a bullet item"}              |
| numbered_list_item   | {"text": "this is a numbered item"}            |

`props` is stored as a JSON blob in SQLite. Prop validity is enforced in code: when reading a row back, the `type` column selects the variant to deserialize into - never infer the type from the JSON payload alone, since payload-only JSON carries no type information.


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
  "props": {
      "text": "do it",
      "checked": false
  },
  "id": "c02fc1d3-db8b-45c5-a222-27595b15aea7",
  "parent_id": "59833787-2cf9-4fdf-8782-e53db20768a5",
  "created_at": "2026-03-01T19:05:00.000Z",
  "updated_at": "2026-07-06T19:41:00.000Z",
}
```

## Tree relationship

The tree is built with queries, not with a self-referencing property on the Block struct

Storage (flat rows, what the DB has):      Tree (built on demand, what you see):
A  parent: NULL                            A
B  parent: A                               ├─ B
C  parent: A                               │  └─ D
D  parent: B                               └─ C

