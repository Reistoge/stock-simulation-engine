# Working with the ORM

for storing and manipulating data between the postgreSQL database, this project uses [toasty-orm](https://tokio-rs.github.io/toasty/nightly/guide/introduction.html) a code-first object relational mapping.

# How can i add a new Entity ?

1. Create a new file **new_entity.rs** inside this folder (src/db/schema)
2. Create the necessary entities properties it should be something like this

```rust
// new_entity.rs
use jiff::Timestamp;

#[derive(Debug, toasty::Model)]
#[table = "new_entity"]
pub struct NewEntity {
    #[key]
    #[auto]
    id: uuid::Uuid,

    #[column(type = varchar(100))]
    name: String,

    #[unique]
    #[column(type = varchar(320))]
    email: String,

    password: String,

    #[has_one]
    profile: toasty::Deferred<Option<Profile>>,

    #[default(Timestamp::now())]
    created_at: Timestamp,
    #[update(Timestamp::now())]
    updated_at: Timestamp,
}
```
* **Note**: See the docs of toasty or similar files inside this directory to see how to create the properties constraints, limits and relationships
    * [Defining Models](https://tokio-rs.github.io/toasty/nightly/guide/defining-models.html)
    * [Field Options](https://tokio-rs.github.io/toasty/nightly/guide/field-options.html)
    * [Relationships](https://tokio-rs.github.io/toasty/nightly/guide/relationships.html)

3. Add the new Entity to the src/db/schema/mod.rs file
```rust
// mod.rs
                   -
pub mod profile;    |
pub mod stock;      |-> entities already added
...                 |
pub mod user;       |
                   -
pub mod new_entity; <-- your new entity !
```
**Note**: This make your entity model visible to the rest of the code.

4. You already added the entity to the Modeling we have to sync the database with the codebase, for this we are gonna use the migration-cli that toasty offers.

### Toasty-cli
5. Run ```cargo run --bin cli -- migration generate```, this will generate the .sql code inside the src/toasty of the new changes
6. Run ```cargo run --bin cli -- migration apply ``` to update the database.

**Note**: For more information about the commands visit the docs about [migrations](https://tokio-rs.github.io/toasty/nightly/guide/schema-management.html).  


# Workflow
1. Edit your model structs (add a field, change a type, add an index)
2. Run ```migration generate --name describe_change```
3. Review the generated SQL file
4. Run ```migration apply to update the database```
5. Commit the migration files, snapshot, and updated history alongside your code


